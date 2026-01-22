//! Common test utilities

use std::f32::consts::PI;
use std::io::Write;
use std::path::Path;

/// Generate a mono sine wave at the given frequency
pub fn generate_sine(freq: f32, sample_rate: u32, duration_secs: f32) -> Vec<f32> {
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    (0..num_samples)
        .map(|i| (2.0 * PI * freq * i as f32 / sample_rate as f32).sin() * 0.5)
        .collect()
}

/// Generate white noise
pub fn generate_noise(sample_rate: u32, duration_secs: f32, seed: u64) -> Vec<f32> {
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    let mut rng = SimpleRng::new(seed);
    (0..num_samples)
        .map(|_| rng.next_f32() * 2.0 - 1.0)
        .collect()
}

/// Simple pseudo-random number generator (xorshift)
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u64() as f64 / u64::MAX as f64) as f32
    }
}

/// Generate a sine wave with amplitude envelope (for dynamics testing)
pub fn generate_sine_with_envelope(
    freq: f32,
    sample_rate: u32,
    duration_secs: f32,
    amplitude_func: impl Fn(f32) -> f32, // time (0-1) -> amplitude (0-1)
) -> Vec<f32> {
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / num_samples as f32;
            let amp = amplitude_func(t);
            (2.0 * PI * freq * i as f32 / sample_rate as f32).sin() * 0.5 * amp
        })
        .collect()
}

/// Generate multi-tone signal (sum of multiple frequencies)
pub fn generate_multitone(
    freqs_and_amps: &[(f32, f32)], // (frequency, amplitude)
    sample_rate: u32,
    duration_secs: f32,
) -> Vec<f32> {
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    (0..num_samples)
        .map(|i| {
            freqs_and_amps
                .iter()
                .map(|(freq, amp)| (2.0 * PI * freq * i as f32 / sample_rate as f32).sin() * amp)
                .sum::<f32>()
                * 0.5
        })
        .collect()
}

/// Write samples as a WAV file to the given path
pub fn write_wav(path: &Path, samples: &[f32], sample_rate: u32) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    write_wav_to(&mut file, samples, sample_rate)
}

/// Write samples as WAV data to a writer
fn write_wav_to<W: Write>(
    writer: &mut W,
    samples: &[f32],
    sample_rate: u32,
) -> std::io::Result<()> {
    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
    let block_align = channels * bits_per_sample / 8;
    let data_size = samples.len() as u32 * 2; // 16-bit = 2 bytes per sample
    let file_size = 36 + data_size;

    // RIFF header
    writer.write_all(b"RIFF")?;
    writer.write_all(&file_size.to_le_bytes())?;
    writer.write_all(b"WAVE")?;

    // fmt chunk
    writer.write_all(b"fmt ")?;
    writer.write_all(&16u32.to_le_bytes())?; // chunk size
    writer.write_all(&1u16.to_le_bytes())?; // PCM format
    writer.write_all(&channels.to_le_bytes())?;
    writer.write_all(&sample_rate.to_le_bytes())?;
    writer.write_all(&byte_rate.to_le_bytes())?;
    writer.write_all(&block_align.to_le_bytes())?;
    writer.write_all(&bits_per_sample.to_le_bytes())?;

    // data chunk
    writer.write_all(b"data")?;
    writer.write_all(&data_size.to_le_bytes())?;

    // Convert f32 samples to i16 and write
    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let i16_sample = (clamped * 32767.0) as i16;
        writer.write_all(&i16_sample.to_le_bytes())?;
    }

    Ok(())
}
