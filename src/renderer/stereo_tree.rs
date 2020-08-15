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
pub struct StereoTree {
  width: u32,
  height: u32,
  hue: f32
}

impl StereoTree {
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

impl RendererBase for StereoTree {
  fn render (&mut self, gl: &mut GlGraphics, context: Context, args: &RenderArgs, audio: &AnalyzedAudio) {

    // Always make sure to use the correct sizes to calculate with
    self.width = args.draw_size[0];
    self.height = args.draw_size[1];
    let center: f64 = self.width as f64 / 2.0;
    let center_bar_width: f64 = self.width as f64 * 0.001; // 0,1 %
    let max_width: f64 = self.width as f64 / 3.0;

    if audio.frequency[0].is_empty() {
      return; // Nothing to render
    }

    // Before we are done rendering, display the center bar
    rectangle([1.0, 1.0, 1.0, 1.0], [center - center_bar_width / 2.0, 0.0, center_bar_width, self.height as f64], context.transform, gl);

    // We don't want 22kHz displayed as this would be WAY too unreasonable,
    // so we need to find the correct cutoff frequency for which to perform
    // some crude lowpass filter.
    // f / bin = i
    let mut cutoff_frequency = (20_000.0 / audio.bin_frequency).floor() as usize;
    if cutoff_frequency > audio.frequency[0].len() {
      cutoff_frequency = audio.frequency[0].len();
    }

    // Determine how high the frequency bars may be at the most
    // let rectangle_width: f64 = (self.width as f64 / 2.0) / cutoff_frequency as f64;
    let frequency_bar_height: f64 = self.height as f64 / cutoff_frequency as f64;
    let amplitude_bar_height: f64 = self.height as f64 / audio.amplitude[0].len() as f64;

    // Second, calculate the maximum frequency amplitude (= max width)
    let mut max_frequency_amp = 0.0;
    for sample in audio.frequency[0][0..cutoff_frequency].iter() {
      if sample.abs() as f64 > max_frequency_amp {
        max_frequency_amp = sample.abs() as f64;
      }
    }
    // Third, calculate the maximum volume amplitude (= max width)
    let mut max_volume_amp = 0.0;
    for sample in audio.amplitude[0].iter() {
      if sample.abs() as f64 > max_volume_amp {
        max_volume_amp = sample.abs() as f64;
      }
    }

    // Display the bars! First the amplitude (as grey underlying bars) ...
    for i in 0..audio.amplitude[0].len() {
      let amplitude_left = audio.amplitude[0][i] as f64;
      let amplitude_right = audio.amplitude[1][i] as f64;

      let mut width_left: f64 = amplitude_left / max_volume_amp; // val from 0.0-1.0
      let mut width_right: f64 = amplitude_right / max_volume_amp;

      // Normalize
      if width_left > 1.0 {
        width_left = 1.0;
      }
      if width_right > 1.0 {
        width_right = 1.0;
      }

      // Transform to final values
      width_left *= max_width;
      width_right *= max_width;

      // Now calculate the rectangles
      let posx_left = center - width_left;
      let posy_left = self.height as f64 - amplitude_bar_height * (i as f64 + 1.0);
      let posx_right = center; // Always begins in the center
      let posy_right = posy_left;

      let opacity = self.hue / 360.0;

      rectangle([0.3, 0.3, 0.3, opacity], [posx_left, posy_left, width_left, amplitude_bar_height], context.transform, gl);
      rectangle([0.3, 0.3, 0.3, opacity], [posx_right, posy_right, width_right, amplitude_bar_height], context.transform, gl);
    }

    // ... and then a colourful frequency on top
    for i in 0..cutoff_frequency {
      let col = Self::hue_to_rgb(self.hue + i as f32 % 360.0);

      let frequency_left = audio.frequency[0][i] as f64;
      let frequency_right = audio.frequency[1][i] as f64;

      let mut width_left: f64 = frequency_left.abs() / max_frequency_amp; // val from 0.0-1.0
      let mut width_right: f64 = frequency_right.abs() / max_frequency_amp;

      // Normalize
      if width_left > 1.0 {
        width_left = 1.0;
      }
      if width_right > 1.0 {
        width_right = 1.0;
      }

      // Transform to final values
      width_left *= max_width;
      width_right *= max_width;

      // Now calculate the rectangles
      let posx_left = center - width_left;
      let posy_left = self.height as f64 - frequency_bar_height * (i as f64 + 1.0);
      let posx_right = center; // Always begins in the center
      let posy_right = posy_left;

      rectangle(col, [posx_left, posy_left, width_left, frequency_bar_height], context.transform, gl);
      rectangle(col, [posx_right, posy_right, width_right, frequency_bar_height], context.transform, gl);
    }

    // And done!
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
