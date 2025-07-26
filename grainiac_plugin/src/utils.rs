use std::path::PathBuf;

pub struct AudioHandler {}

impl AudioHandler {
    pub fn open(path: PathBuf) -> Option<Vec<f32>> {
        if let Ok(mut reader) = hound::WavReader::open(path) {
            let spec = reader.spec();

            let num_channels = spec.channels as usize;

            let raw_samples: Vec<f32> = match spec.sample_format {
                hound::SampleFormat::Int => {
                    if spec.bits_per_sample == 16 {
                        reader
                            .samples::<i16>()
                            .map(|s| s.unwrap() as f32 / i16::MAX as f32)
                            .collect()
                    } else if spec.bits_per_sample == 24 {
                        // hound doesn't directly support i24, usually stored as i32
                        reader
                            .samples::<i32>()
                            .map(|s| s.unwrap() as f32 / (2_i32.pow(23) as f32)) // normalize 24-bit range
                            .collect()
                    } else {
                        return None;
                    }
                }
                hound::SampleFormat::Float => reader.samples::<f32>().map(|s| s.unwrap()).collect(),
            };

            // Convert to mono by averaging across channels per frame
            let mut mono_samples = Vec::with_capacity(raw_samples.len() / num_channels);
            for frame in raw_samples.chunks(num_channels) {
                let sum: f32 = frame.iter().copied().sum();
                mono_samples.push(sum / num_channels as f32);
            }

            return Some(mono_samples);
        }

        None
    }

    //pub fn create_preview(samples: Vec<f32>) {}
}
