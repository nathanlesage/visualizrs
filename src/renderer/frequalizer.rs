use crate::traits::RendererBase;
use piston::input::{UpdateArgs, RenderArgs, Key};
use opengl_graphics::GlGraphics;
use graphics::Context;

use graphics::{rectangle};

use crate::audio::AnalyzedAudio;

/**
 * Each renderer consists of three things. First, the struct defining its
 * state. Secondly, an impl that defines the specific methods of the struct
 * that won't be called by the application. And third, the trait implementation
 * which defines all methods that are necessary as the application expects them.
 */
pub struct Frequalizer {
  width: u32,
  height: u32,
  hue: f32
}

impl Frequalizer {
  pub fn create () -> Self {
    Self {
      width: 200,
      height: 200,
      hue: 0.0
    }
  }

  /// Helper function to calculate a color based on the hue
  fn hue_to_rgb (hue: f32) -> [f32; 4] {
    let hue_segment = (hue as f64 / 60.0).trunc() as u32 + 1; // Get one of the six segments
    let fraction: f32 = hue % 60.0 / 60.0;

    let mut rgba: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

    // LOOK AT MA MATH SKILLZ!!
    match hue_segment {
      1 => {
        rgba[0] = 1.0;
        rgba[1] = fraction;
        rgba[2] = 0.0;
      },
      2 => {
        rgba[0] = fraction;
        rgba[1] = 1.0;
        rgba[2] = 0.0;
      },
      3 => {
        rgba[0] = 0.0;
        rgba[1] = 1.0;
        rgba[2] = fraction;
      },
      4 => {
        rgba[0] = 0.0;
        rgba[1] = fraction;
        rgba[2] = 1.0;
      },
      5 => {
        rgba[0] = fraction;
        rgba[1] = 0.0;
        rgba[2] = 1.0;
      },
      _ => {
        rgba[0] = 1.0;
        rgba[1] = 0.0;
        rgba[2] = fraction;
      } // Includes segment 6 and weird other numbers
    }

    rgba
  }
}

impl RendererBase for Frequalizer {
  fn render (&mut self, gl: &mut GlGraphics, context: Context, args: &RenderArgs, audio: &AnalyzedAudio) {

    // Always make sure to use the correct sizes to calculate with
    self.width = args.draw_size[0];
    self.height = args.draw_size[1];

    if audio.frequency[0].is_empty() {
      return; // Nothing to render
    }

    let mut moving_hue = self.hue;

    // We don't want 22kHz displayed as this would be WAY too unreasonable,
    // so we need to find the correct cutoff frequency for which to perform
    // some crude lowpass filter.
    // f / bin = i
    let mut cutoff = (20_000.0 / audio.bin_frequency).floor() as usize;
    if cutoff > audio.frequency[0].len() {
      cutoff = audio.frequency[0].len();
    }

    let rectangle_width: f64 = self.width as f64 / cutoff as f64;

    // Second, calculate the maximum number (= max height)
    let mut max_amplitude = 0.0;
    for sample in audio.frequency[0][0..cutoff].iter() {
      if sample.abs() > max_amplitude {
        max_amplitude = sample.abs();
      }
    }

    let max_amplitude = max_amplitude as f64;

    for i in 0..cutoff {
      moving_hue += 1.0;
      if moving_hue > 360.0 {
        moving_hue = 0.0;
      }

      let col = Self::hue_to_rgb(moving_hue);

      let amplitude_left = audio.frequency[0][i] as f64;
      let amplitude_right = audio.frequency[0][i] as f64;
      let mut degree_left: f64 = amplitude_left / max_amplitude; // val from 0.0-1.0
      let mut degree_right: f64 = amplitude_right / max_amplitude;
      if degree_left > 1.0 {
        degree_left = 1.0;
      }
      if degree_right > 1.0 {
        degree_right = 1.0;
      }

      let posx = i as f64 * rectangle_width;
      let posy_left = (1.0 - degree_left) * self.height as f64;
      let posy_right = (1.0 - degree_right) * self.height as f64;
      let height_left = degree_left * self.height as f64;
      let height_right = degree_right * self.height as f64;

      rectangle(col, [posx, posy_left, rectangle_width, height_left], context.transform, gl);
      rectangle(col, [posx, posy_right, rectangle_width, height_right], context.transform, gl);
    }

    // Next, display the waveform in form of points from center
    let center_screen: f64 = self.height as f64 / 2.0;
    let ratio = self.height as f64 * 0.2;
    for i in 0..audio.amplitude[0].len() {
      rectangle([1.0, 1.0, 1.0, 1.0], [i as f64 * rectangle_width, center_screen + audio.amplitude[0][i] as f64 * ratio, rectangle_width, rectangle_width], context.transform, gl);
    }
  }

  fn update (&mut self, _args: &UpdateArgs) {
    self.hue += 0.1;
    if self.hue > 360.0 {
      self.hue = 0.0;
    }
  }

  fn on_cursor_movement (&mut self, _x: f64, _y: f64) {
    // This renderer does not react to mouse events :(
  }

  fn on_cursor_state (&mut self, _is_over_window: bool) {
    // This renderer does not react to mouse events :(
  }

  fn on_click (&mut self) {
    // Don't react to anything
  }

  fn on_keypress (&mut self, _key: Key) {
    // Stoic renderer, I tell you
  }
}
