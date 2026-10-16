use cpal::{
    Device, Host, SampleFormat, Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use rtrb::{Producer, RingBuffer};

const AUDIO_SAMPLE_RATE: u32 = 44_100;
const AUDIO_CHANNELS: u16 = 2;
const AUDIO_BUFFER_CAPACITY: usize = 8192;

pub struct AudioOutput {
    _host: Host,
    _device: Device,
    _stream: Stream,
    producer: Producer<(f32, f32)>,
}

impl AudioOutput {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .ok_or_else(|| "No audio output device available".to_string())?;

        let mut supported_configs = device
            .supported_output_configs()
            .map_err(|error| format!("Failed to query audio configurations: {error}"))?;

        let supported_config = supported_configs
            .find(|config| {
                config.channels() == AUDIO_CHANNELS
                    && config.sample_format() == SampleFormat::F32
                    && config.min_sample_rate() <= AUDIO_SAMPLE_RATE
                    && config.max_sample_rate() >= AUDIO_SAMPLE_RATE
            })
            .ok_or_else(|| {
                "No stereo audio configuration supports 44100 Hz with f32 samples".to_string()
            })?;

        let stream_config = supported_config
            .with_sample_rate(AUDIO_SAMPLE_RATE)
            .config();

        let (producer, mut consumer) = RingBuffer::new(AUDIO_BUFFER_CAPACITY);

        let stream = device
            .build_output_stream(
                stream_config,
                move |output: &mut [f32], _| {
                    output.fill(0.0);

                    let (frames, _) = output.as_chunks_mut::<2>();

                    for frame in frames {
                        if let Ok((left, right)) = consumer.pop() {
                            frame[0] = left;
                            frame[1] = right;
                        }
                    }
                },
                |error| {
                    eprintln!("Audio stream error: {error}");
                },
                None,
            )
            .map_err(|error| format!("Failed to build audio stream: {error}"))?;

        stream
            .play()
            .map_err(|error| format!("Failed to start audio stream: {error}"))?;

        Ok(Self {
            _host: host,
            _device: device,
            _stream: stream,
            producer,
        })
    }

    pub fn push_samples(&mut self, samples: Vec<(f32, f32)>) {
        for sample in samples {
            if self.producer.push(sample).is_err() {
                // The buffer is full; discard samples rather than blocking.
                break;
            }
        }
    }
}
