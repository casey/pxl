use std::sync::{Arc, Mutex};

use {runtime::{cpal::{self, EventLoop, Format, SampleRate, StreamData,
                      UnknownTypeOutputBuffer, SupportedFormat, Sample},
               error::Error},

     Synthesizer,
     SAMPLES_PER_SECOND};


pub struct Speaker {
  synthesizer: Arc<Mutex<Synthesizer>>,
  event_loop: EventLoop,
}

impl Speaker {
  pub fn new(synthesizer: Arc<Mutex<Synthesizer>>) -> Result<Speaker, Error> {
    let event_loop = EventLoop::new();

    let device = cpal::default_output_device().ok_or(Error::AudioOutputDeviceInitialization)?;

    let mut supported_output_formats = device.supported_output_formats()
      .map_err(|_| Error::AudioOutputDeviceInitialization)?
      .filter(|f| {
        f.channels == 2
          && f.min_sample_rate <= SampleRate(SAMPLES_PER_SECOND)
          && f.max_sample_rate >= SampleRate(SAMPLES_PER_SECOND)
      }).collect::<Vec<SupportedFormat>>();

    supported_output_formats.sort_unstable_by(|a, b| a.cmp_default_heuristics(b));

    let supported_output_format = supported_output_formats.first()
      .ok_or(Error::AudioOutputDoesNotSupport48khzSampleRate)?;

    let output_format = Format {
      channels:    2,
      sample_rate: SampleRate(SAMPLES_PER_SECOND),
      data_type:   supported_output_format.data_type,
    };

    let stream_id = event_loop.build_output_stream(&device, &output_format).unwrap();

    event_loop.play_stream(stream_id);

    Ok(Speaker {
      synthesizer,
      event_loop,
    })
  }

  pub fn play(self) -> ! {
    let synthesizer = self.synthesizer;
    let event_loop = self.event_loop;
    let mut samples = Vec::new();
    let mut samples_played = 0;

    event_loop.run(move |_stream_id, stream_data| {
      if let StreamData::Output { buffer } = stream_data {
        let sample_count = buffer.len() / 2;
        samples.clear();
        samples.resize(
          sample_count,
          ::Sample {
            left: 0.0,
            right: 0.0,
          },
        );
        synthesizer
          .lock()
          .unwrap()
          .synthesize(samples_played, &mut samples);
        samples_played += sample_count as u64;
        match buffer {
          UnknownTypeOutputBuffer::F32(mut buffer) => {
            let mut i = 0;
            for sample in samples.iter() {
              buffer[i + 0] = sample.left;
              buffer[i + 1] = sample.right;
              i += 2;
            }
          }
          UnknownTypeOutputBuffer::I16(mut buffer) => {
            let mut i = 0;
            for sample in samples.iter() {
              buffer[i + 0] = sample.left.to_i16();
              buffer[i + 1] = sample.right.to_i16();
              i += 2;
            }
          }
          UnknownTypeOutputBuffer::U16(mut buffer) => {
            let mut i = 0;
            for sample in samples.iter() {
              buffer[i + 0] = sample.left.to_u16();
              buffer[i + 1] = sample.right.to_u16();
              i += 2;
            }
          }
        }
      } else {
        panic!("unexpected audio input stream");
      }
    });
  }
}
