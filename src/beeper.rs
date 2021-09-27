use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream, StreamConfig,
};

pub struct Beeper {
    stream: Option<Stream>,
    shared_state_ptr: Arc<AtomicBool>,
    previous_state: bool,
}

// @TODO handle errors
impl Beeper {
    pub fn new() -> Self {
        let initial_state = false;
        let shared_state_ptr = Arc::new(AtomicBool::new(initial_state));

        Self {
            stream: None,
            shared_state_ptr,
            previous_state: initial_state,
        }
    }

    pub fn start_stream(&mut self) {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();
        let state_ptr = self.shared_state_ptr.clone();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => make_stream::<f32>(state_ptr, &device, &config.into()),
            cpal::SampleFormat::I16 => make_stream::<i16>(state_ptr, &device, &config.into()),
            cpal::SampleFormat::U16 => make_stream::<u16>(state_ptr, &device, &config.into()),
        };
        stream.play().unwrap();

        self.stream = Some(stream)
    }

    pub fn stop_stream(&mut self) {
        self.stream = None
    }

    pub fn set_beeper_active(&mut self, new_state: bool) {
        if self.previous_state != new_state {
            self.previous_state = new_state;
            self.shared_state_ptr
                .store(self.previous_state, Ordering::Relaxed);
        }
    }
}

fn make_stream<T>(
    shared_state_ptr: Arc<AtomicBool>,
    device: &cpal::Device,
    config: &StreamConfig,
) -> cpal::Stream
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sine of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut sinewave_value_fn = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };
    let mut silence_value_fn = || 0.0;

    device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                if shared_state_ptr.load(Ordering::Relaxed) {
                    write_data(data, channels, &mut sinewave_value_fn)
                } else {
                    write_data(data, channels, &mut silence_value_fn)
                }
            },
            |err| eprintln!("an error occurred on stream: {}", err),
        )
        .unwrap()
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
