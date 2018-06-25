use std::sync::{Arc, Mutex};

use {runtime::{cpal::{self, EventLoop, Format, SampleFormat, SampleRate, StreamData,
                      UnknownTypeOutputBuffer},
               error::Error},

     Synthesizer,
     Sample,
     SAMPLES_PER_SECOND};


pub struct Speaker {
  synthesizer: Arc<Mutex<Synthesizer>>,
  event_loop: EventLoop,
}

impl Speaker {
  pub fn new(synthesizer: Arc<Mutex<Synthesizer>>) -> Result<Speaker, Error> {
    let event_loop = EventLoop::new();

    let device = cpal::default_output_device().ok_or(Error::AudioOutputDeviceInitialization)?;

    let format = Format {
      channels: 2,
      data_type: SampleFormat::F32,
      sample_rate: SampleRate(SAMPLES_PER_SECOND),
    };

    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();

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
          Sample {
            left: 0.0,
            right: 0.0,
          },
        );
        synthesizer
          .lock()
          .unwrap()
          .synthesize(samples_played, &mut samples);
        samples_played += sample_count as u64;
        if let UnknownTypeOutputBuffer::F32(mut buffer) = buffer {
          let mut i = 0;
          for sample in samples.iter() {
            buffer[i + 0] = sample.left;
            buffer[i + 1] = sample.right;
            i += 2;
          }
        } else {
          panic!("unexpected audio output stream format");
        }
      } else {
        panic!("unexpected audio input stream");
      }
    });
  }
}
