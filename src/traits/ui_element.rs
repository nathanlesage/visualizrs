use piston::input::{
  UpdateArgs,
  RenderArgs,
  Key
};

use opengl_graphics::GlGraphics;
use graphics::Context;

pub enum UIEvent {
  RequestChangeAudioDevice(usize),
  Selection(usize, usize),
  RequestChangeRenderer(usize)
}

pub trait UIElement {
  // Specific traits for UI Elements
  fn render (&mut self, gl: &mut GlGraphics, context: Context, args: &RenderArgs);
  fn update (&mut self, args: &UpdateArgs);

  /// Called whenever the cursor position changed
  fn on_cursor_movement (&mut self, x: f64, y: f64);

  /// Called whenever the cursor enters or leaves the window
  fn on_cursor_state (&mut self, is_over_window: bool);

  /// Called when the left button has been pressed
  fn on_click (&mut self) -> Option<UIEvent>;

  /// Called when a key on the keyboard has been pressed
  fn on_keypress (&mut self, key: Key);
}
