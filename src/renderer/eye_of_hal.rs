use crate::traits::RendererBase;
use piston::input::{UpdateArgs, RenderArgs, Key};
use opengl_graphics::GlGraphics;
use graphics::{Context, Transformed};

use graphics::rectangle;

use crate::audio::AnalyzedAudio;

/**
 * Each renderer consists of three things. First, the struct defining its
 * state. Secondly, an impl that defines the specific methods of the struct
 * that won't be called by the application. And third, the trait implementation
 * which defines all methods that are necessary as the application expects them.
 */
pub struct EyeOfHAL {
  width: u32,
  height: u32,
  hue: u32,
  zoom_factor: f64,
  zoom_dir: f64,
  max_zoom: f64,
  min_zoom: f64
}

impl EyeOfHAL {
  pub fn create () -> Self {
    Self {
      width: 200,
      height: 200,
      hue: 0,
      zoom_factor: 0.4,
      zoom_dir: 0.01,
      max_zoom: 0.45,
      min_zoom: 0.35
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

impl RendererBase for EyeOfHAL {
  fn render (&mut self, gl: &mut GlGraphics, context: Context, args: &RenderArgs, audio: &AnalyzedAudio) {

    // Always make sure to use the correct sizes to calculate with
    self.width = args.draw_size[0];
    self.height = args.draw_size[1];

    let mut moving_hue = self.hue as f32;

    // We don't want 22kHz displayed as this would be WAY too unreasonable,
    // so we need to find the correct cutoff frequency for which to perform
    // some crude lowpass filter.
    // f / bin = i
    let mut cutoff = (20_000.0 / audio.bin_frequency).floor() as usize;
    if cutoff > audio.frequency[0].len() {
      cutoff = audio.frequency[0].len();
    }

    let mut max_height = 0.0;
    if self.height > self.width {
      max_height = self.height as f64 * 0.7;
    } else {
      max_height = self.width as f64 * 0.7;
    }

    // Second, calculate the maximum number (= max height)
    let mut max_amplitude = 0.0;
    for sample in audio.frequency[0][0..cutoff].iter() {
      if sample.abs() > max_amplitude {
        max_amplitude = sample.abs();
      }
    }

    let max_amplitude = max_amplitude as f64;

    let pos_x = self.width as f64 / 2.0;
    let pos_y = self.height as f64 / 2.0;

    let centered_matrix = context.transform.trans(self.width as f64 / 2.0, self.height as f64 / 2.0);

    for i in 0..cutoff {
      moving_hue += 1.0;
      if moving_hue > 360.0 {
        moving_hue = 0.0;
      }

      let deg = i as f64 / cutoff as f64 * 360.0;

      let col = Self::hue_to_rgb(moving_hue);
      let mut degree: f64 = audio.frequency[0][i] as f64 / max_amplitude; // val from 0.0-1.0
      if degree > 1.0 {
        degree = 1.0;
      }
      let height = degree * max_height;

      rectangle(col, [pos_x, pos_y, 5.0, height], centered_matrix.zoom(self.zoom_factor).rot_rad(deg), gl);
    }

    if audio.channels > 1 {
      for i in 0..cutoff {
        moving_hue += 1.0;
        if moving_hue > 360.0 {
          moving_hue = 0.0;
        }

        let deg = i as f64 / cutoff as f64 * 360.0;
        let deg = deg + 2.0; // offset to mono channel

        let col = Self::hue_to_rgb(moving_hue);
        let mut degree: f64 = audio.frequency[1][i] as f64 / max_amplitude; // val from 0.0-1.0
        if degree > 1.0 {
          degree = 1.0;
        }
        let height = degree * max_height;

        rectangle(col, [pos_x, pos_y, 5.0, height], centered_matrix.zoom(self.zoom_factor).rot_rad(deg), gl);
      }
    }

    for (i, sample) in audio.amplitude[0].iter().enumerate() {
      let deg = i as f64 / audio.amplitude[0].len() as f64 * 360.0;
      rectangle(Self::hue_to_rgb(self.hue as f32), [0.0, 0.0, 1.0, sample.abs() as f64 * 600.0], centered_matrix.rot_rad(deg), gl);
    }
  }

  fn update (&mut self, _args: &UpdateArgs) {
    // Also update the hue
    self.hue += 1;
    if self.hue > 360 {
      self.hue = 0;
    }

    self.zoom_factor += self.zoom_dir;
    if self.zoom_factor > self.max_zoom {
      self.zoom_dir = -0.00001;
    } else if self.zoom_factor < self.min_zoom {
      self.zoom_dir = 0.00001;
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
