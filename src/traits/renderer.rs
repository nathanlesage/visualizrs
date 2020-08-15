use piston::input::{
  UpdateArgs,
  RenderArgs,
  Key
};

use opengl_graphics::GlGraphics;
use graphics::Context;
use crate::audio::AnalyzedAudio;

/**
 * This defines the contract the application expects from the renderer objects.
 * This way we can ensure one, that numerous renderers can be created, and
 * second that all of them have this minimal interface for the application to
 * work with.
 */
pub trait RendererBase {
  fn render (&mut self, gl: &mut GlGraphics, context: Context, args: &RenderArgs, audio: &AnalyzedAudio);
  fn update (&mut self, args: &UpdateArgs);

  /// Called whenever the cursor position changed
  fn on_cursor_movement (&mut self, x: f64, y: f64);

  /// Called whenever the cursor enters or leaves the window
  fn on_cursor_state (&mut self, is_over_window: bool);

  /// Called when the left button has been pressed
  fn on_click (&mut self);

  /// Called when a key on the keyboard has been pressed
  fn on_keypress (&mut self, key: Key);
}
