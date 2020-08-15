// Import the rendering contract
use crate::traits::RendererBase;
use crate::traits::UIElement;
use crate::traits::UIEvent;

// We make use of the arguments of the event loop
use piston::input::{UpdateArgs, RenderArgs, Key};

// Used to send events to the application
use std::sync::mpsc;

// Use the GlGraphics backend
use opengl_graphics::GlGraphics;
use graphics::character::CharacterCache;

// Import drawing helper functions
use graphics::{Context, rectangle, text, line, Transformed};

// Font imports
use opengl_graphics::GlyphCache;
use opengl_graphics::TextureSettings;

// Needed to satisfy the trait
use crate::audio::AnalyzedAudio;

// Import UI elements
mod dropdown;
use dropdown::UIDropdown;
mod util;

use util::{cursor_in_rect, format_number};

// Needed for the timeout
use std::time;

// Needed to resolve the asset path
use std::env::current_dir;

use std::path::Path;

// Necessary to retrieve a list of available input devices.
use crate::audio::util::{fetch_devices, AudioDevice};

static AUDIO_IO_ID: usize = 1;
static RENDERER_ID: usize = 2;

pub struct UI<'a> {
  width: u32,
  height: u32,
  last_cursor_x: f64,
  last_cursor_y: f64,
  cursor_over_window: bool,
  should_display_ui: bool,
  menu_display_time: u64, // in seconds
  mouse_last_moved: time::Instant,
  ui_opacity: f64,
  target_opacity: f64,
  ui_font: graphics::glyph_cache::rusttype::GlyphCache<'a, (), opengl_graphics::Texture>,
  // Displayable settings
  available_devices: Vec<AudioDevice>,
  available_renderers: Vec<String>,
  ui_elements: Vec<Box<dyn UIElement>>,
  selected_device: usize,
  selected_renderer: usize,
  event_sender: Option<mpsc::Sender<UIEvent>>,
  device_info: String,
  base_font_size: f64,
  font_path: String,
  input_selector_button_rect: [f64; 4],
  renderer_selector_button_rect: [f64; 4],
  input_selector_index: i32,
  renderer_selector_index: i32,
  min_amp: f32,
  max_amp: f32
}

impl UI<'static> {
  pub fn create () -> Self {
    let __dirname = current_dir().unwrap(); // When in development-mode, it'll refer to the crate's root, not the actual binary
    let path_to_font = "assets/fonts/lato/Lato-Regular.ttf";
    let font_path = String::from(Path::new(&__dirname).join(path_to_font).to_str().unwrap());
    println!("{}", font_path);
    let glyph_cache = GlyphCache::new(font_path.clone(), (), TextureSettings::new()).unwrap();

    Self {
      // General window parameters
      width: 0,
      height: 0,
      last_cursor_x: 0.0,
      last_cursor_y: 0.0,
      cursor_over_window: false,
      mouse_last_moved: time::Instant::now(),
      // General UI parameters
      should_display_ui: false, // If true, will increase the ui_opacity to the target
      ui_opacity: 0.0, // Will increase as long as display var is true, else decrease
      target_opacity: 0.7, // The final opacity of the UI when fully shown
      menu_display_time: 2,
      ui_font: glyph_cache,
      // Information
      available_devices: fetch_devices(),
      available_renderers: Vec::new(),
      selected_device: 0,
      selected_renderer: 0,
      event_sender: None,
      device_info: String::from("No device selected"),
      font_path,
      ui_elements: Vec::new(),
      base_font_size: 12.0,
      input_selector_button_rect: [0.0, 0.0, 0.0, 0.0],
      renderer_selector_button_rect: [0.0, 0.0, 0.0, 0.0],
      input_selector_index: -1,
      renderer_selector_index: -1,
      min_amp: 0.0,
      max_amp: 0.0
    }
  }

  // Helper and utility functions
  pub fn selected_audio_device_changed (&mut self, idx: usize) {
    self.selected_device = idx;
  }

  pub fn selected_renderer_changed (&mut self, idx: usize) {
    self.selected_renderer = idx;
  }

  pub fn register_action_callback (&mut self, tx: mpsc::Sender<UIEvent>) {
    self.event_sender = Some(tx);
  }

  pub fn set_available_renderers (&mut self, rend: Vec<String>) {
    self.available_renderers = rend;
  }

  /// Draw a text button and return the actual rectangle where it has been drawn
  fn draw_text_button (&mut self, begin_point: [f64; 2], text: String, gl: &mut GlGraphics, context: Context) -> [f64; 4] {
    // Draws a text button with the UIs style
    let padding = 5.0;
    let real_width = self.ui_font.width(self.base_font_size as u32, text.as_str()).unwrap() + 2.0 * padding;
    let real_height = self.base_font_size + 2.0 * padding;

    let rect = [
      begin_point[0],
      begin_point[1],
      begin_point[0] + real_width,
      begin_point[1] + real_height
    ];

    // Hover effect
    // if cursor_in_rect([self.last_cursor_x, self.last_cursor_y], self.input_selector_button_rect) {
      let line_color = [0.9, 0.9, 0.9, self.ui_opacity as f32];
      // Four lines surrounding the button
      line(line_color, 0.5, [rect[0], rect[1], rect[2], rect[1]], context.transform, gl);
      line(line_color, 0.5, [rect[2], rect[1], rect[2], rect[3]], context.transform, gl);
      line(line_color, 0.5, [rect[2], rect[3], rect[0], rect[3]], context.transform, gl);
      line(line_color, 0.5, [rect[0], rect[3], rect[0], rect[1]], context.transform, gl);
    // }

    // Now the text within it
    let fg_color = [1.0, 1.0, 1.0, self.ui_opacity as f32];
    text::Text::new_color(fg_color, self.base_font_size as u32).draw(
      text.as_str(),
      &mut self.ui_font,
      &context.draw_state,
      context.transform.trans(begin_point[0] + padding, begin_point[1] + padding + self.base_font_size),
      gl
    ).unwrap();

    // Finally return the actual rectangle
    [
      begin_point[0],
      begin_point[1],
      real_width,
      real_height
    ]
  }
}

impl RendererBase for UI<'static> {
  fn render (&mut self, gl: &mut GlGraphics, context: Context, args: &RenderArgs, audio: &AnalyzedAudio) {
    if self.ui_opacity == 0.0 {
      return // If the opacity is zero, we don't need to waste resources
    }

    // Window size
    self.width = args.draw_size[0];
    self.height = args.draw_size[1];

    // Overlay size (width is always full)
    let overlay_top = self.height as f64 * 0.8;
    let overlay_height = self.height as f64 * 0.2;

    // Font size relative to UI overlay (always three lines high)
    self.base_font_size = (overlay_height / 3.0 * 0.95).floor();
    if self.base_font_size > 14.0 {
      self.base_font_size = 14.0; // Don't overdo it
    }

    // Colors
    let bg_color = [0.0, 0.0, 0.0, self.ui_opacity as f32];

    // Overlay area
    let overlay_rect = [
      0.0,
      overlay_top,
      self.width as f64,
      overlay_height
    ];

    // Draw the overlay
    rectangle(bg_color, overlay_rect, context.transform, gl);

    let mut selected_device = String::from("No device selected");
    // Check if we have a device selected
    if !self.available_devices.is_empty() && self.selected_device < self.available_devices.len() {
      selected_device = self.available_devices[self.selected_device].name.clone();
    }

    self.device_info = format!("IN: {}", selected_device);

    // Draw the input selection button
    self.input_selector_button_rect = self.draw_text_button([10.0, overlay_rect[1] + 10.0], self.device_info.clone(), gl, context);

    // ... and the renderer
    self.renderer_selector_button_rect = self.draw_text_button(
      [10.0 + self.input_selector_button_rect[2] + 20.0, overlay_rect[1] + 10.0],
      format!("Renderer: {}", self.available_renderers[self.selected_renderer].clone()),
      gl, context);

    let fg_color = [1.0, 1.0, 1.0, self.ui_opacity as f32];

    // Draw a small spectrogram to indicate whether audio is actually being received
    let amp_bar_height = self.renderer_selector_button_rect[3] as f32;
    let start_x = self.renderer_selector_button_rect[0] + self.renderer_selector_button_rect[2] + 10.0;
    let start_y = self.renderer_selector_button_rect[1] + self.renderer_selector_button_rect[3];
    let w = 50.0 / audio.amplitude[0].len() as f64;
    for (i, sample) in audio.amplitude[0].iter().enumerate() {
      let h = (sample.abs() * amp_bar_height) as f64;
      rectangle(fg_color, [start_x + i as f64 * w, start_y - h, w, h], context.transform, gl);
    }

    // Now provide audio information in the next lines
    let padding = 5.0;

    // Sample rate
    text::Text::new_color(fg_color, self.base_font_size as u32).draw(
      format!("Sample rate: {} Hz", format_number(audio.sample_rate as f64)).as_str(),
      &mut self.ui_font,
      &context.draw_state,
      context.transform.trans(10.0 + padding, overlay_rect[1] + 10.0 + self.base_font_size * 2.0 + 3.0 * padding),
      gl
    ).unwrap();

    // Buffer size
    text::Text::new_color(fg_color, self.base_font_size as u32).draw(
      format!("Buffer size: {} samples", format_number(audio.buffer_size as f64)).as_str(),
      &mut self.ui_font,
      &context.draw_state,
      context.transform.trans(10.0 + padding, overlay_rect[1] + 10.0 + self.base_font_size * 3.0 + 4.0 * padding),
      gl
    ).unwrap();

    let mut max_frequency = 0.0;
    for sample in audio.frequency[0].clone() {
      if sample > max_frequency {
        max_frequency = sample;
      }
      if sample < self.min_amp {
        self.min_amp = sample
      }
      if sample > self.max_amp {
        self.max_amp = sample
      }
    }

    // Min/max frequency
    // Buffer size
    text::Text::new_color(fg_color, self.base_font_size as u32).draw(
      format!(
        "Analyzed frequencies: {} Hz to {} Hz (channels: {})",
        format_number(audio.bin_frequency.round() as f64),
        format_number((audio.bin_frequency * audio.frequency[0].len() as f32).round() as f64),
        audio.channels
      ).as_str(),
      &mut self.ui_font,
      &context.draw_state,
      context.transform.trans(10.0 + padding, overlay_rect[1] + 10.0 + self.base_font_size * 4.0 + 5.0 * padding),
      gl
    ).unwrap();

    let mut items = Vec::new();
    for device in self.available_devices.iter() {
      items.push(device.name.clone());
    }

    // Now display all UI elements
    for elem in self.ui_elements.iter_mut() {
      elem.render(gl, context, args);
    }
  }

  fn update (&mut self, _args: &UpdateArgs) {
    let now = time::Instant::now();
    if now.duration_since(self.mouse_last_moved) > time::Duration::new(self.menu_display_time, 0) {
      self.should_display_ui = false;
    }

    // Adapt the animation
    if !self.should_display_ui && self.ui_opacity > 0.0 {
      self.ui_opacity -= 0.1;
    } else if self.should_display_ui && self.ui_opacity < self.target_opacity {
      self.ui_opacity += 0.1;
    }
  }

  fn on_cursor_movement (&mut self, x: f64, y: f64) {
    self.last_cursor_x = x;
    self.last_cursor_y = y;
    self.mouse_last_moved = time::Instant::now();
    self.should_display_ui = true;

    // Now propagate to all UI elements
    for elem in self.ui_elements.iter_mut() {
      elem.on_cursor_movement(x, y);
    }
  }

  fn on_cursor_state (&mut self, is_over_window: bool) {
    self.cursor_over_window = is_over_window;
    if !is_over_window {
      self.should_display_ui = false;
    }
  }

  fn on_click (&mut self) {
    // Check for generated events on the UI Elements
    // Now propagate to all UI elements
    for elem in self.ui_elements.iter_mut() {
      if let Some(event) = elem.on_click() {
        if let UIEvent::Selection(idx, id) = event {
          // Send event to application
          if self.event_sender.is_some() && id == AUDIO_IO_ID {
            self.event_sender.as_ref().unwrap().send(UIEvent::RequestChangeAudioDevice(idx)).unwrap();
          } else if self.event_sender.is_some() && id == RENDERER_ID {
            // let event = match idx {
            //   1 => {
            //     RendererType::Circle
            //   },
            //   2 => {
            //     RendererType::Tree
            //   },
            //   _ => { RendererType::Square } // Everything 0 and non-covered
            // };

            self.event_sender.as_ref().unwrap().send(UIEvent::RequestChangeRenderer(idx)).unwrap();
          }
        }
      }
    }

    // Display the dropdown if the cursor is currently in the input device selector button rect
    if cursor_in_rect(
      [self.last_cursor_x, self.last_cursor_y],
      self.input_selector_button_rect
    ) && self.input_selector_index < 0 {
        let mut items = Vec::new();
        for device in self.available_devices.iter() {
          items.push((device.index, device.name.clone()));
        }

        self.ui_elements.push(
          Box::new(
            UIDropdown::create(
              AUDIO_IO_ID,
              items, true,
              [self.input_selector_button_rect[0], self.input_selector_button_rect[1]],
              self.input_selector_button_rect[2],
              self.base_font_size,
              self.font_path.clone()
            )
          )
        );
        // Save the index for later
        self.input_selector_index = self.ui_elements.len() as i32 - 1;
    } else if self.input_selector_index > -1 && !self.ui_elements.is_empty() {
        // Remove that thing again
        self.ui_elements.remove(self.input_selector_index as usize);
        self.input_selector_index = -1;
      }

    if cursor_in_rect([self.last_cursor_x, self.last_cursor_y],
    self.renderer_selector_button_rect) && self.renderer_selector_index < 0 {
      let mut items = Vec::new();
        for (i, renderer) in self.available_renderers.iter().enumerate() {
          items.push((i, renderer.clone()));
        }

        self.ui_elements.push(
          Box::new(
            UIDropdown::create(
              RENDERER_ID,
              items, true,
              [self.renderer_selector_button_rect[0], self.renderer_selector_button_rect[1]],
              self.renderer_selector_button_rect[2],
              self.base_font_size,
              self.font_path.clone()
            )
          )
        );
        // Save the index for later
        self.input_selector_index = self.ui_elements.len() as i32 - 1;
      } else if self.renderer_selector_index > -1 && !self.ui_elements.is_empty() {
        self.ui_elements.remove(self.renderer_selector_index as usize);
        self.renderer_selector_index = -1;
      }
    }

  fn on_keypress (&mut self, _key: Key) {
    // ...
  }
}
