// Audio utility functions -- basically nice to have
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SupportedStreamConfigRange, StreamConfig};
use std::sync::mpsc;

// Store information about available audio devices for easy switching
pub struct AudioDevice {
  pub index: usize,
  pub name: String,
  // Save all supported configurations for easy access through the GUI
  pub supported_configurations: Vec<SupportedStreamConfigRange>
}

/// Fetches all audio devices available on the default host
pub fn fetch_devices () -> Vec<AudioDevice> {
  // Get the default host, e.g. CoreAudio, Jack etc.
  let host = cpal::default_host();

  // Save all available devices into our buffer
  let all_devices = host.input_devices().expect("Could not get a list of available input devices!");
  let mut ret = Vec::new();

  for (device_index, device) in all_devices.enumerate() {
    // There is at least one input configuration available;
    // let's retrieve all of them
    let input_configs = device.supported_input_configs().unwrap(); // We can be sure it's NOT an error
    let mut cfg = Vec::new();
    for (_idx, config) in input_configs.enumerate() {
      cfg.push(config);
    }

    // Make the AudioDevice struct
    ret.push(AudioDevice {
      index: device_index,
      name: device.name().unwrap(),
      supported_configurations: cfg
    })
  }

  ret // Return the devices
}

/// Creates an input listening stream and returns both the stream and the data receiver
pub fn create_stream (device_index: Option<usize>, device_config: Option<StreamConfig>) -> (impl StreamTrait, StreamConfig, mpsc::Receiver<std::vec::Vec<f32>>, usize) {
  // The input stream will live in a different thread, so we need a transmitter
  // to safely transmit data to this (main) thread
  let (tx, rx) = mpsc::channel();

  // Get the default host, e.g. CoreAudio, Jack etc.
  let host = cpal::default_host();

  // By default, use the default input device, unless the user has provided a device_index

  let mut real_idx = 0;

  let device = match device_index {
    Some (idx) => {
      let mut all_input_devices = host.input_devices().expect("Could not get a list of available input devices!");
      real_idx = idx;
      all_input_devices.nth(idx).unwrap_or_else(|| {
        real_idx = 0;
        all_input_devices.nth(real_idx).unwrap()
      })
    },
    None => {
      host.default_input_device().expect("No device!")
    }
  };

  // Now some debug output stuff etc.
  let name = device.name();
  println!("Listening to device {} ...", name.unwrap());

  let supported_configs_range = device.supported_input_configs()
    .expect("error while querying configs");

  for (idx, config) in supported_configs_range.enumerate() {
    println!("Supported config {}: {:?}", idx, config);
  }

  let config = match device_config {
    Some (cfg) => { cfg },
    // Just use the default
    None => { device.supported_input_configs().unwrap().next().unwrap().with_max_sample_rate().config() }
  };

  let stream = device.build_input_stream(
    &config,
    move |data: &[f32], _: &cpal::InputCallbackInfo| {
      // Here's when we get a full buffer.
      // We need to take ownership of the full slice which
      // we are doing with a vector, and send it to the
      // main thread
      let mut safe_buffer = Vec::new();
      for elem in data {
        safe_buffer.push(*elem);
      }
      tx.send(safe_buffer).unwrap();
    },
    move |err| {
      println!("ERROR: {}", err);
    },
  ).unwrap();

  stream.play().expect("Could not start stream!");
  (stream, config, rx, real_idx) // Return the stream and the receiver
}
