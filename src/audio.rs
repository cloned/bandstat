use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use std::fs::File;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Target sample rate for analysis (ensures consistent FFT resolution)
pub(crate) const TARGET_SAMPLE_RATE: u32 = 48000;

pub(crate) struct AudioData {
    pub(crate) samples: Vec<f32>,
    pub(crate) sample_rate: u32,
    pub(crate) channels: u16,
    pub(crate) original_sample_rate: u32,
}

pub(crate) fn load_audio(filename: &str) -> Result<AudioData, String> {
    let file = File::open(filename).map_err(|e| format!("{}: {}", filename, e))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
    {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("{}: unsupported format ({})", filename, e))?;

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or_else(|| format!("{}: no audio track found", filename))?;

    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or_else(|| format!("{}: unknown sample rate", filename))?;
    let channels = track
        .codec_params
        .channels
        .ok_or_else(|| format!("{}: unknown channel count", filename))?
        .count() as u16;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("{}: failed to create decoder ({})", filename, e))?;

    let track_id = track.id;
    let mut samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(e) => return Err(format!("{}: error reading packet ({})", filename, e)),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Warning: decode error: {}", e);
                continue;
            }
        };

        let spec = *decoded.spec();
        let num_channels = spec.channels.count();

        let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        sample_buf.copy_interleaved_ref(decoded);

        for chunk in sample_buf.samples().chunks(num_channels) {
            let mono: f32 = chunk.iter().sum::<f32>() / num_channels as f32;
            samples.push(mono);
        }
    }

    // Resample to target sample rate if needed
    let (final_samples, final_sample_rate) = if sample_rate != TARGET_SAMPLE_RATE {
        let resampled = resample(&samples, sample_rate, TARGET_SAMPLE_RATE)?;
        (resampled, TARGET_SAMPLE_RATE)
    } else {
        (samples, sample_rate)
    };

    Ok(AudioData {
        samples: final_samples,
        sample_rate: final_sample_rate,
        channels,
        original_sample_rate: sample_rate,
    })
}

fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Result<Vec<f32>, String> {
    // Fast settings suitable for analysis (not mastering quality)
    let params = SincInterpolationParameters {
        sinc_len: 64,
        f_cutoff: 0.91,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 128,
        window: WindowFunction::Hann,
    };

    let ratio = to_rate as f64 / from_rate as f64;
    let chunk_size = 4096;

    let mut resampler = SincFixedIn::<f32>::new(ratio, 2.0, params, chunk_size, 1)
        .map_err(|e| format!("Failed to create resampler: {}", e))?;

    let mut output = Vec::with_capacity((samples.len() as f64 * ratio) as usize + chunk_size);
    let mut chunk = vec![0.0f32; chunk_size];
    let mut pos = 0;

    while pos < samples.len() {
        let end = (pos + chunk_size).min(samples.len());
        let len = end - pos;

        chunk[..len].copy_from_slice(&samples[pos..end]);
        // Zero-pad remainder if last chunk is short
        if len < chunk_size {
            chunk[len..].fill(0.0);
        }

        let input = [chunk.as_slice()];
        let resampled = resampler
            .process(&input, None)
            .map_err(|e| format!("Resample error: {}", e))?;

        output.extend_from_slice(&resampled[0]);
        pos += chunk_size;
    }

    // Trim to expected length
    let expected_len = (samples.len() as f64 * ratio) as usize;
    output.truncate(expected_len);

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    /// Generate a sine wave at the given frequency
    fn generate_sine(freq: f32, sample_rate: u32, duration_secs: f32) -> Vec<f32> {
        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        (0..num_samples)
            .map(|i| (2.0 * PI * freq * i as f32 / sample_rate as f32).sin())
            .collect()
    }

    #[test]
    fn test_resample_output_length() {
        // 44100 -> 48000: ratio = 48000/44100 ≈ 1.0884
        let input = generate_sine(440.0, 44100, 1.0);
        let output = resample(&input, 44100, 48000).unwrap();

        let expected_len = (input.len() as f64 * 48000.0 / 44100.0) as usize;
        assert_eq!(output.len(), expected_len);
    }

    #[test]
    fn test_resample_preserves_frequency() {
        // Generate 1000Hz sine at 44100Hz, resample to 48000Hz
        // The resampled signal should still have peaks at ~1000Hz
        let freq = 1000.0;
        let input = generate_sine(freq, 44100, 0.1);
        let output = resample(&input, 44100, 48000).unwrap();

        // Count zero crossings to estimate frequency
        let zero_crossings: usize = output
            .windows(2)
            .filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0))
            .count();

        // Expected crossings: 2 per cycle * freq * duration
        let expected_crossings = (2.0 * freq * 0.1) as usize;
        let tolerance = expected_crossings / 10; // 10% tolerance

        assert!(
            (zero_crossings as i32 - expected_crossings as i32).unsigned_abs() < tolerance as u32,
            "Expected ~{} zero crossings, got {}",
            expected_crossings,
            zero_crossings
        );
    }

    #[test]
    fn test_resample_downsample() {
        // 96000 -> 48000: ratio = 0.5
        let input = generate_sine(440.0, 96000, 0.5);
        let output = resample(&input, 96000, 48000).unwrap();

        let expected_len = (input.len() as f64 * 0.5) as usize;
        assert_eq!(output.len(), expected_len);
    }

    #[test]
    fn test_resample_same_rate() {
        // Edge case: same rate should work (though load_audio skips this)
        let input = generate_sine(440.0, 48000, 0.1);
        let output = resample(&input, 48000, 48000).unwrap();

        assert_eq!(output.len(), input.len());
    }
}
