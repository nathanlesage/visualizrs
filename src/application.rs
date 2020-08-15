// Holds the application, basically this holds everything together

// Std imports
use std::time;
use std::sync::mpsc;
use mpsc::TryRecvError;

// Imaging imports
use opengl_graphics::{GlGraphics};

// Window imports
use piston::window::WindowSettings;
use glutin_window::GlutinWindow;
use piston::{AdvancedWindow, Size}; // AdvancedWindow to get and set the window title

// Needed for the PhysicalSize struct
use winit::dpi;

// We need an infinite event loop
use piston::event_loop::{EventSettings, Events};
// ... and events to react to
use piston::input::{
  RenderEvent,
  RenderArgs,
  UpdateEvent,
  UpdateArgs,
  ResizeEvent,
  ResizeArgs,
  MouseCursorEvent,
  CursorEvent,
  ReleaseEvent,
  Button,
  MouseButton,
  Key
};

// Renderers
use super::traits::RendererBase;
use super::renderer::Frequalizer;
use super::renderer::EyeOfHAL;
use super::renderer::StereoTree;
use super::user_interface::UI;
use super::traits::UIEvent;

// Audio in/out
use super::audio::{Audio, AudioEvent};

// The tuple-array for the renderer selection dropdown
const AVAILABLE_RENDERERS: [(&str, usize); 3] = [
  ("Frequalizer", 0),
  ("The Eye of HAL", 1),
  ("StereoTree", 3)
];

const WINDOW_TITLE: &str = "VisualizRS";

pub struct App<'a>{
  gl: GlGraphics, // OpenGL drawing backend
  window: GlutinWindow, // Containing Window
  // We wrap the renderers in a Box, because this way we basically say "Well,
  // we don't care which renderer you use, as long as it supports this trait"
  // (which equals "As long as it supports the renderer contract")
  renderer: Box<dyn RendererBase>, // A customizable renderer
  user_interface: UI<'a>, // The user interface
  frame_counter: u32,
  last_check: time::Instant,
  audio_io: Audio,
  ui_action_rx: Option<mpsc::Receiver<UIEvent>>,
  audio_action_rx: Option<mpsc::Receiver<AudioEvent>>
}

impl App<'static> {
  pub fn boot (ver: glutin_window::OpenGL) -> App<'static> {

    // First, define the Window settings
    let settings = WindowSettings::new(WINDOW_TITLE, Size::from([640, 480]))
    .graphics_api(ver) // Use OpenGL API
    .exit_on_esc(true); // DEBUG remove for production!

    // Now build a new window
    let window: GlutinWindow = settings.build().expect("Could not create window");

    // Now instantiate all modules
    let gl_instance = GlGraphics::new(ver);

    let mut instance = App {
      gl: gl_instance,
      window,
      renderer: Box::new(Frequalizer::create()), // Default boring renderer!
      user_interface: UI::create(), // Default non-handler
      frame_counter: 0,
      last_check: time::Instant::now(),
      audio_io: Audio::create(),
      ui_action_rx: None,
      audio_action_rx: None
    };

    // Tell the UI the available renderers
    let mut rend = Vec::new();
    for renderer in AVAILABLE_RENDERERS.iter() {
      rend.push(String::from(renderer.0));
    }
    instance.user_interface.set_available_renderers(rend);

    // Set up the communication channel between the user interface and the application
    let (tx, rx) = mpsc::channel();
    instance.ui_action_rx = Some(rx);
    instance.user_interface.register_action_callback(tx);

    // ... and for the audio system
    let (tx, rx) = mpsc::channel();
    instance.audio_action_rx = Some(rx);
    instance.audio_io.register_action_callback(tx);
    instance // Finally return the instance
  }

  fn render(&mut self, args: &RenderArgs) {
    self.frame_counter += 1;

    // This beautiful renderer of us consists basically of three steps:
    // 1. Clear out the context
    // 2. Let the selected renderer do its thing
    // 3. Render the user interface last in order to have it supersede the rendered stuff

    let renderer = &mut self.renderer;
    let ui = &mut self.user_interface;
    let audio_data = self.audio_io.get_analyzed_audio();

    self.gl.draw(args.viewport(), |c, gl| {
      use graphics::{clear};
      // Clear the screen with black (necessary first step!)
      clear([0.0, 0.0, 0.0, 1.0], gl);
      renderer.render(gl, c, args, &audio_data);
      ui.render(gl, c, args, &audio_data);
    });
  }

  /// Updates the application every args.dt milliseconds
  fn update(&mut self, args: &UpdateArgs) {
    let now = time::Instant::now();
    if now.duration_since(self.last_check) > time::Duration::new(1, 0) {
      self.window.set_title(format!("{} ({} fps)", WINDOW_TITLE, self.frame_counter));
      self.last_check = now;
      self.frame_counter = 0;
    }

    self.renderer.update(args);
    self.user_interface.update(args);

    // Also make sure to fetch new audio to be consumed on every update
    self.audio_io.fetch_new_audio();

    // Are there user interface events in the receiving end that we should handle?
    match self.ui_action_rx.as_ref().unwrap().try_recv() {
      Ok(event) => {
        match event {
          UIEvent::RequestChangeAudioDevice(idx) => {
            self.audio_io.switch_device(idx);
          },
          // We shall hot-swap the renderer
          UIEvent::RequestChangeRenderer(id) => {
            match id {
              1 => {
                self.renderer = Box::new(EyeOfHAL::create());
                self.user_interface.selected_renderer_changed(1);
              },
              2 => {
                self.renderer = Box::new(StereoTree::create());
                self.user_interface.selected_renderer_changed(2);
              },
              _ => {
                // If uncovered, take this
                self.renderer = Box::new(Frequalizer::create());
                self.user_interface.selected_renderer_changed(0);
              }
            }
          },
          _ => { /* */ }
        }
      },
      Err(TryRecvError::Empty) => { /* All good, no new events available, continue as we were */ },
      Err(TryRecvError::Disconnected) => {
        // TODO: Reconnect to stream if possible
        println!("The user interface has hung up on us >:O");
      }
    }

    // Are there audio events in the receiving end that we should handle?
    match self.audio_action_rx.as_ref().unwrap().try_recv() {
      Ok(event) => {
        match event {
          AudioEvent::InputDeviceChanged(idx) => {
            println!("A new audio input device has been selected: {}!", idx);
            self.user_interface.selected_audio_device_changed(idx);
          }
        }
      },
      Err(TryRecvError::Empty) => { /* All good, no new events available, continue as we were */ },
      Err(TryRecvError::Disconnected) => {
        println!("The audio system has hung up on us >:O");
      }
    }
  }

  fn resize(&mut self, args: &ResizeArgs) {
    // From the context docs:
    // Some platforms (macOS, Wayland) require being manually updated when their window or surface is resized.
    // The easiest way of doing this is to take every Resized window event that is received with a LogicalSize
    // and convert it to a PhysicalSize and pass it into this function.

    // Cf: https://docs.rs/glutin/0.17.0/glutin/dpi/index.html
    // let dpi = self.window.window().get_hidpi_factor();
    // TODO: Somehow doesn't work on macOS built-in retina displays
    self.window.ctx.resize(
      dpi::PhysicalSize::from_logical(
        (args.viewport().window_size[0], args.viewport().window_size[1]), 2.0
      )
      // dpi::PhysicalSize::new(
      //   args.viewport().window_size[0], args.viewport().window_size[1]
      // )
    );
  }

  fn cursor_position (&mut self, args: &[f64; 2]) {
    self.renderer.on_cursor_movement(args[0], args[1]);
    self.user_interface.on_cursor_movement(args[0], args[1]);
  }

  fn cursor_state (&mut self, args: bool) {
    self.renderer.on_cursor_state(args);
    self.user_interface.on_cursor_state(args);
  }

  fn on_click (&mut self, button: MouseButton) {
    if button == MouseButton::Left {
      self.user_interface.on_click();
      self.renderer.on_click();
    }
  }

  fn on_keypress (&mut self, _key: Key) {
    //
  }

  pub fn main_loop(&mut self) {
    // Defines the general loop
    let mut events = Events::new(EventSettings::new());

    // events.next will always wait for a new event, unless the user
    // wishes to close the app (which is when events.next will return
    // None, which doesn't satisfy the while-condition, hence this
    // function ends and hence the main()).
    while let Some(e) = events.next(&mut self.window) {
      // We should render
      if let Some(args) = e.render_args() {
        self.render(&args);
      }

      // We should update (runs every x milliseconds)
      if let Some(args) = e.update_args() {
        self.update(&args);
      }

      // On macOS and Wayland, we need to manually update the context's size
      if let Some(args) = e.resize_args() {
        self.resize(&args);
      }

      if let Some(args) = e.cursor_args() {
        // Is the cursor in or out of the application?
        self.cursor_state(args);
      }

      if let Some(args) = e.mouse_cursor_args() {
        self.cursor_position(&args);
      }

      if let Some(button) = e.release_args() {
        match button {
          Button::Keyboard(key) => self.on_keypress(key),
          Button::Mouse(button) => self.on_click(button),
          // Button::Controller(button) => println!("Released controller button '{:?}'", button),
          // Button::Hat(hat) => println!("Released controller hat `{:?}`", hat),
          _ => { /* Unknown event, we don't care */ }
        }
      };
    }
  }
}
