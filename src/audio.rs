/// Audio input/output handler

use cpal::traits::{StreamTrait};
use std::sync::mpsc;
use mpsc::TryRecvError;

// FFT imports
use rustfft::FFT;
use rustfft::algorithm::Radix4;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

pub mod util;

use util::{create_stream};

pub enum AudioEvent {
  InputDeviceChanged(usize) // Emitted with the new device index
}

#[derive(Clone)] // Derive the clone ability, because all fields of this struct are clonable
pub struct AnalyzedAudio {
  pub amplitude: [Vec<f32>; 2], // The original PCM amplitude buffer (sample size)
  pub frequency: [Vec<f32>; 2], // The analyzed frequency amplitudes (half the sample size)
  pub buffer_size: u32, // Buffer length
  pub sample_rate: u32, // The sample rate, e.g. 44,100 Hz
  pub bin_frequency: f32, // The frequency of the bins, e.g. 43Hz for sampling 44.1kHz at 1,024 buffer size
  pub channels: usize, // The amount of channels we're recording with (mono or stereo)
}

// Example directory: https://github.com/RustAudio/cpal/blob/master/examples
pub struct Audio {
  // Holds the stream (necessary to retrieve samples)
  stream: Option<Box<dyn StreamTrait>>,
  // Last read buffer slice
  last_buffer: AnalyzedAudio,
  thread_recv: mpsc::Receiver<std::vec::Vec<f32>>,
  event_sender: Option<mpsc::Sender<AudioEvent>>,
  // Necessary info for the current stream
  sample_rate: u32,
  buffer_size: usize,
  channels: usize, // Holds the channel number
  // bpm: usize // TODO: Actually calculate the bpm at some point
}

impl Audio {
  pub fn create () -> Self {
    // The input stream will live in a different thread, so we need a transmitter
    // to safely transmit data to this (main) thread
    let (stream, config, rx, _real_idx) = create_stream(Some(0), None); // By default, use the first device (not default, b/c we can't extract the device index)

    let audio_buf = AnalyzedAudio {
      amplitude: [Vec::new(), Vec::new()],
      frequency: [Vec::new(), Vec::new()],
      sample_rate: config.sample_rate.0, // Pry the sample rate out of this struct
      bin_frequency: 0.0,
      channels: config.channels as usize,
      buffer_size: 0
    };

    Self {
      stream: Some(Box::new(stream)),
      last_buffer: audio_buf, // Initialize with empty buffer
      thread_recv: rx, // Save the receiver
      event_sender: None, // Used by the application to receive audio events
      buffer_size: 0, // Will be set after the first sample set has been received b/c the default buffer size can be difficult
      sample_rate: config.sample_rate.0,
      // bpm: 0,
      channels: config.channels as usize
    }
  } // END constructor

  /// Switches to a different device.
  pub fn switch_device (&mut self, device_index: usize) {
    let (stream, config, rx, real_index) = create_stream(Some(device_index), None);
    self.sample_rate = config.sample_rate.0;
    self.stream = Some(Box::new(stream));
    self.thread_recv = rx;
    if self.event_sender.is_some() {
      self.event_sender.as_ref().unwrap().send(AudioEvent::InputDeviceChanged(real_index)).unwrap();
    }
  }

  /// Registers an event transmitter to receive feedback on some changes in the audio system
  pub fn register_action_callback (&mut self, tx: mpsc::Sender<AudioEvent>) {
    self.event_sender = Some(tx);
  }

  fn analyze (&mut self, buf: Vec<f32>) {
    // Why don't we take this from the config? B/c the default buffer size is
    // undefined, but we'll know it as soon as we have the sample
    // NOTE: This buffer is INTERLEAVED, that means LRLRLRLRLRLR ...!!!
    self.buffer_size = buf.len() / self.channels; // len for mono, len/2 for stereo

    let mut buffers: [Vec<f32>; 2] = [
      Vec::new(),
      Vec::new()
    ]; // If it's mono, we'll duplicate the full buffer

    if self.channels == 2 {
      // This basically omits all channels over 2
      for i in 0..self.buffer_size {
        buffers[0].push(buf[i * self.channels]);
        buffers[1].push(buf[i * self.channels + 1]);
      }
    } else if self.channels == 1 {
      for sample in buf.iter() {
        buffers[0].push(*sample);
        buffers[1].push(*sample);
      }
    }

    // Perform FFT on both buffers. For computational reason,
    // first on the first, and if we have mono data, also just
    // clone it, instead of running the FFT again.
    let mut output: [Vec<f32>; 2] = [
      Vec::new(),
      Vec::new()
    ];

    output[0] = self.run_fft(&buffers[0]);
    if self.channels == 2 {
      // Run the FFT on the right channel as well
      output[1] = self.run_fft(&buffers[1]);
    } else {
      // Copy the left channel
      output[1] = output[0].clone();
    }

    self.last_buffer = AnalyzedAudio {
      amplitude: buffers,
      frequency: output,
      sample_rate: self.sample_rate,
      bin_frequency: self.sample_rate as f32 / self.buffer_size as f32,
      channels: self.channels,
      buffer_size: self.buffer_size as u32
    };
  }

  /// Runs an FFT run on an audio buffer for one channel.
  fn run_fft (&mut self, buf: &[f32]) -> Vec<f32> {
    let mut input: Vec<Complex<f32>> = Vec::new();
    for sample in buf.iter() {
      input.push(Complex::from(*sample));
    }

    // We also need the reference value (e.g. the highest amplitude)
    let mut max_amplitude: f32 = 0.0;
    for sample in buf.iter() {
      if sample.abs() > max_amplitude {
        max_amplitude = *sample;
      }
    }

    // output must be equal to input, so we'll just prepopulate with zeroes
    let mut output: Vec<Complex<f32>> = vec![Complex::zero(); input.len()];

    // We use Radix4, as audio buffers are always a power of 2. And if not
    // we know the drill: Pad with zeroes!
    let fft = Radix4::new(input.len(), false);
    fft.process(&mut input, &mut output); // Aaaaaand GO!

    let mut output: Vec<f32> = output.iter().map(| el | {
      // So, what we have in the output currently is a set of complex numbers which represent both the phase (which we
      // don't need) and the magnitude/amplitude at the given frequency (which we want). There are multiple solutions, apparently,
      // but what we actually want is sqrt(a^2 + b^2), so let's do that. Luckily, the complex number provides us with exactly that function:
      el.norm_sqr().sqrt() // Important: We have to call sqrt later on ourselves!
    }).collect();

    output.drain(0..output.len() / 2 - 1); // now it should be buf.len / 4

    // Cut off the 22,050Hz frequency (in case of 44.1kHz and 1,024 buffer size)
    // as well as the DC point
    output[0] = 0.0;
    let last_idx = output.len() - 1;
    output[last_idx] = 0.0;

    // And return
    output
  }

  pub fn fetch_new_audio (&mut self) {
    match self.thread_recv.try_recv() {
      Ok(buf) => self.analyze(buf),
      Err(TryRecvError::Empty) => { /* All good, no buffer data available, continue as we were */ },
      Err(TryRecvError::Disconnected) => {
        // TODO: Reconnect to stream if possible
        println!("The remote thread has terminated!");
      }
    }
  }

  pub fn get_analyzed_audio (&mut self) -> AnalyzedAudio {
    self.last_buffer.clone() // Return a clone of the buffer
  }
}

impl Drop for Audio {
  fn drop (&mut self) {
    println!("Audio shutting down ...");
    if self.stream.is_some() {
      self.stream.as_ref().unwrap().pause().expect("Could not stop stream!");
    }
  }
}
