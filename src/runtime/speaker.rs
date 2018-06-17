use std::sync::{Arc, Mutex};

use ::{
  SAMPLES_PER_SECOND, Sample, Program,
  runtime::{
    error::Error,
    cpal::{self, EventLoop, Format, SampleFormat, SampleRate, StreamData, UnknownTypeOutputBuffer},
  },
};

pub struct Speaker {
  program: Arc<Mutex<Program>>,
  event_loop: EventLoop,
}

impl Speaker {
  pub fn new(program: Arc<Mutex<Program>>) -> Result<Speaker, Error> {
      let event_loop = EventLoop::new();

      let device =
        cpal::default_output_device().ok_or(Error::AudioOutputDeviceInitialization)?;

      let format = Format {
        channels: 2,
        data_type: SampleFormat::F32,
        sample_rate: SampleRate(SAMPLES_PER_SECOND),
      };

      let stream_id = event_loop
        .build_output_stream(&device, &format)
        .unwrap();

      event_loop.play_stream(stream_id);

      Ok(Speaker {
        program,
        event_loop,
      })
  }

  pub fn play(self) -> ! {
    let program = self.program;
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
        program
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
