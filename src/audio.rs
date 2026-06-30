use anyhow::{Result, Context};
use std::sync::{Arc, Mutex};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct AudioMonitor {
    amplitude: Arc<Mutex<f32>>,
}

impl AudioMonitor {
    pub fn new() -> Result<(Self, cpal::Stream)> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .context("No input device found")?;

        let config: cpal::StreamConfig = device.default_input_config()
            .context("Failed to get default input config")?
            .into();

        let amplitude = Arc::new(Mutex::new(0.0));
        let amplitude_clone = Arc::clone(&amplitude);

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut max_amp = 0.0;
                for &sample in data {
                    let abs_sample = sample.abs();
                    if abs_sample > max_amp {
                        max_amp = abs_sample;
                    }
                }
                if let Ok(mut amp) = amplitude_clone.lock() {
                    *amp = max_amp;
                }
            },
            |err| {
                log::error!("Audio stream error: {}", err);
            },
            None
        )?;

        stream.play()?;

        Ok((Self {
            amplitude,
        }, stream))
    }

    pub fn get_amplitude(&self) -> f32 {
        *self.amplitude.lock().expect("Amplitude mutex poisoned")
    }
}

unsafe impl Send for AudioMonitor {}
unsafe impl Sync for AudioMonitor {}
