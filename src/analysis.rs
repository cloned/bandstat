use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::Arc;

use crate::audio::AudioData;

pub(crate) const FFT_SIZE: usize = 4096;

/// Minimum power threshold to avoid log(0) in dB calculations
const MIN_POWER: f64 = 1e-20;

/// Dynamics threshold in dB below band peak (frames below this are considered inaudible)
const DYNAMICS_THRESHOLD_DB: f64 = 60.0;

/// Minimum band power percentage to display dynamics (bands below this show "-")
pub(crate) const DYNAMICS_DISPLAY_THRESHOLD_PCT: f64 = 0.5;

pub(crate) struct Band {
    pub(crate) label: &'static str,
    pub(crate) low_hz: f32,
    pub(crate) high_hz: f32,
}

pub(crate) fn get_bands() -> Vec<Band> {
    vec![
        Band {
            label: "DC",
            low_hz: 0.0,
            high_hz: 20.0,
        },
        Band {
            label: "SUB1",
            low_hz: 20.0,
            high_hz: 40.0,
        },
        Band {
            label: "SUB2",
            low_hz: 40.0,
            high_hz: 60.0,
        },
        Band {
            label: "BASS",
            low_hz: 60.0,
            high_hz: 120.0,
        },
        Band {
            label: "UBAS",
            low_hz: 120.0,
            high_hz: 250.0,
        },
        Band {
            label: "LMID",
            low_hz: 250.0,
            high_hz: 500.0,
        },
        Band {
            label: "MID",
            low_hz: 500.0,
            high_hz: 1000.0,
        },
        Band {
            label: "UMID",
            low_hz: 1000.0,
            high_hz: 2000.0,
        },
        Band {
            label: "HMID",
            low_hz: 2000.0,
            high_hz: 4000.0,
        },
        Band {
            label: "PRES",
            low_hz: 4000.0,
            high_hz: 6000.0,
        },
        Band {
            label: "BRIL",
            low_hz: 6000.0,
            high_hz: 10000.0,
        },
        Band {
            label: "HIGH",
            low_hz: 10000.0,
            high_hz: 14000.0,
        },
        Band {
            label: "UHIG",
            low_hz: 14000.0,
            high_hz: 18000.0,
        },
        Band {
            label: "AIR",
            low_hz: 18000.0,
            high_hz: f32::MAX,
        },
    ]
}

pub(crate) fn create_hanning_window(size: usize) -> Vec<f32> {
    let pi2 = 2.0 * std::f32::consts::PI;
    (0..size)
        .map(|i| 0.5 * (1.0 - (pi2 * i as f32 / (size - 1) as f32).cos()))
        .collect()
}

/// K-weighting filter frequency response (ITU-R BS.1770-4)
/// Coefficients:
/// - 48kHz: ITU-R BS.1770-4 Table 1
/// - 44.1kHz: derived via bilinear transform (cf. pyloudnorm, libebur128)
fn k_weight(freq: f64, sample_rate: f64) -> f64 {
    use std::f64::consts::PI;

    if freq <= 0.0 {
        return 0.0;
    }

    let omega = 2.0 * PI * freq / sample_rate;
    let cos_w = omega.cos();
    let sin_w = omega.sin();
    let cos_2w = (2.0 * omega).cos();
    let sin_2w = (2.0 * omega).sin();

    // Pre-filter (shelving) biquad coefficients from ITU-R BS.1770-4
    let (b0_pre, b1_pre, b2_pre, a1_pre, a2_pre) = if (sample_rate - 48000.0).abs() < 1.0 {
        (
            1.53512485958697,
            -2.69169618940638,
            1.19839281085285,
            -1.69065929318241,
            0.73248077421585,
        )
    } else {
        // 44100Hz coefficients
        (
            1.5308412300503478,
            -2.6509799951547297,
            1.1690790799215869,
            -1.6636551132560204,
            0.7125954280732254,
        )
    };

    let pre_num_re = b0_pre + b1_pre * cos_w + b2_pre * cos_2w;
    let pre_num_im = -b1_pre * sin_w - b2_pre * sin_2w;
    let pre_den_re = 1.0 + a1_pre * cos_w + a2_pre * cos_2w;
    let pre_den_im = -a1_pre * sin_w - a2_pre * sin_2w;
    let pre_mag_sq = (pre_num_re * pre_num_re + pre_num_im * pre_num_im)
        / (pre_den_re * pre_den_re + pre_den_im * pre_den_im);

    // RLB (high-pass) biquad coefficients
    let (b0_rlb, b1_rlb, b2_rlb, a1_rlb, a2_rlb) = if (sample_rate - 48000.0).abs() < 1.0 {
        (1.0, -2.0, 1.0, -1.99004745483398, 0.99007225036621)
    } else {
        // 44100Hz coefficients
        (
            0.9994908682456236,
            -1.9989817364912472,
            0.9994908682456236,
            -1.9989817364912472,
            0.9989826099040272,
        )
    };

    let rlb_num_re = b0_rlb + b1_rlb * cos_w + b2_rlb * cos_2w;
    let rlb_num_im = -b1_rlb * sin_w - b2_rlb * sin_2w;
    let rlb_den_re = 1.0 + a1_rlb * cos_w + a2_rlb * cos_2w;
    let rlb_den_im = -a1_rlb * sin_w - a2_rlb * sin_2w;
    let rlb_mag_sq = (rlb_num_re * rlb_num_re + rlb_num_im * rlb_num_im)
        / (rlb_den_re * rlb_den_re + rlb_den_im * rlb_den_im);

    (pre_mag_sq * rlb_mag_sq).sqrt()
}

/// Check if the sample rate has optimized K-weighting coefficients.
/// Returns a warning message if not.
pub(crate) fn check_sample_rate(sample_rate: u32) -> Option<String> {
    if matches!(sample_rate, 48000 | 44100) {
        None
    } else {
        Some(format!(
            "Warning: K-weighting coefficients are optimized for 48kHz/44.1kHz. \
             Using approximate values for {}Hz.",
            sample_rate
        ))
    }
}

pub(crate) fn create_k_weight_table(fft_size: usize, sample_rate: u32) -> Vec<f64> {
    let freq_per_bin = sample_rate as f64 / fft_size as f64;
    let sr = sample_rate as f64;
    (0..fft_size / 2)
        .map(|bin| {
            let freq = bin as f64 * freq_per_bin;
            let weight = k_weight(freq, sr);
            weight * weight // squared for power spectrum
        })
        .collect()
}

pub(crate) fn analyze_interval(
    samples: &[f32],
    fft: &Arc<dyn rustfft::Fft<f32>>,
    window: &[f32],
    bands: &[Band],
    freq_per_bin: f32,
    k_weights: Option<&[f64]>,
) -> Vec<f64> {
    let nyquist_bin = FFT_SIZE / 2;
    let hop_size = FFT_SIZE / 2;
    let mut band_powers = vec![0.0f64; bands.len()];
    let mut pos = 0;

    while pos + FFT_SIZE <= samples.len() {
        let mut buffer: Vec<Complex<f32>> = (0..FFT_SIZE)
            .map(|j| Complex::new(samples[pos + j] * window[j], 0.0))
            .collect();

        fft.process(&mut buffer);

        for (band_idx, band) in bands.iter().enumerate() {
            let low_bin = ((band.low_hz / freq_per_bin) as usize).min(nyquist_bin);
            let high_bin = ((band.high_hz / freq_per_bin) as usize).min(nyquist_bin);

            let power: f64 = buffer[low_bin..high_bin]
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let bin_power = c.norm_sqr() as f64;
                    match k_weights {
                        Some(weights) => bin_power * weights[low_bin + i],
                        None => bin_power,
                    }
                })
                .sum();

            band_powers[band_idx] += power;
        }

        pos += hop_size;
    }

    band_powers
}

pub(crate) fn powers_to_percentages(powers: &[f64]) -> Vec<f64> {
    let total: f64 = powers.iter().sum();
    if total > 0.0 {
        powers.iter().map(|p| (p / total) * 100.0).collect()
    } else {
        vec![0.0; powers.len()]
    }
}

/// Result of unified stats analysis
pub(crate) struct StatsResult {
    pub(crate) raw_powers: Vec<f64>,
    pub(crate) k_powers: Vec<f64>,
    pub(crate) dynamics: Vec<f64>,
}

/// Analyze all stats in a single FFT pass with optional progress callback
pub(crate) fn analyze_stats<F>(
    audio: &AudioData,
    bands: &[Band],
    k_weights: &[f64],
    mut on_progress: F,
) -> StatsResult
where
    F: FnMut(u8),
{
    let freq_per_bin = audio.sample_rate as f32 / FFT_SIZE as f32;
    let window = create_hanning_window(FFT_SIZE);
    let nyquist_bin = FFT_SIZE / 2;
    let hop_size = FFT_SIZE / 2;

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);

    let mut raw_powers = vec![0.0f64; bands.len()];
    let mut k_powers = vec![0.0f64; bands.len()];
    let mut band_db_per_frame: Vec<Vec<f64>> = vec![Vec::new(); bands.len()];

    let total_frames = if audio.samples.len() >= FFT_SIZE {
        (audio.samples.len() - FFT_SIZE) / hop_size + 1
    } else {
        0
    };

    let mut pos = 0;
    let mut frame_idx = 0;
    let mut last_progress: u8 = 0;

    while pos + FFT_SIZE <= audio.samples.len() {
        let mut buffer: Vec<Complex<f32>> = (0..FFT_SIZE)
            .map(|j| Complex::new(audio.samples[pos + j] * window[j], 0.0))
            .collect();

        fft.process(&mut buffer);

        for (band_idx, band) in bands.iter().enumerate() {
            let low_bin = ((band.low_hz / freq_per_bin) as usize).min(nyquist_bin);
            let high_bin = ((band.high_hz / freq_per_bin) as usize).min(nyquist_bin);

            let mut raw_power = 0.0f64;
            let mut k_power = 0.0f64;

            for (i, c) in buffer[low_bin..high_bin].iter().enumerate() {
                let bin_power = c.norm_sqr() as f64;
                raw_power += bin_power;
                k_power += bin_power * k_weights[low_bin + i];
            }

            raw_powers[band_idx] += raw_power;
            k_powers[band_idx] += k_power;

            // Collect dB for dynamics (using raw power)
            if raw_power > MIN_POWER {
                band_db_per_frame[band_idx].push(10.0 * raw_power.log10());
            }
        }

        // Progress update
        frame_idx += 1;
        if total_frames > 0 {
            let progress = ((frame_idx * 100) / total_frames) as u8;
            if progress != last_progress {
                on_progress(progress);
                last_progress = progress;
            }
        }

        pos += hop_size;
    }

    // Calculate dynamics (standard deviation of dB values)
    // Filter out frames below threshold from band's max (inaudible in normal playback)
    let dynamics: Vec<f64> = band_db_per_frame
        .iter()
        .map(|dbs| {
            if dbs.is_empty() {
                return f64::NEG_INFINITY;
            }

            let max_db = dbs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let threshold = max_db - DYNAMICS_THRESHOLD_DB;

            // Filter frames above threshold
            let filtered: Vec<f64> = dbs.iter().cloned().filter(|&db| db >= threshold).collect();

            if filtered.is_empty() {
                return f64::NEG_INFINITY;
            }

            let n = filtered.len() as f64;
            let mean = filtered.iter().sum::<f64>() / n;
            let variance = filtered.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
            variance.sqrt()
        })
        .collect();

    StatsResult {
        raw_powers,
        k_powers,
        dynamics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Calculate standard deviation of a slice (for testing)
    fn std_dev(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
        variance.sqrt()
    }

    #[test]
    fn test_hanning_window_endpoints() {
        let window = create_hanning_window(1024);
        // Endpoints should be 0
        assert!(window[0].abs() < 1e-6, "First sample should be ~0");
        assert!(
            window[window.len() - 1].abs() < 1e-6,
            "Last sample should be ~0"
        );
    }

    #[test]
    fn test_hanning_window_center() {
        let window = create_hanning_window(1024);
        // Center should be 1
        let center = window[512];
        assert!(
            (center - 1.0).abs() < 0.01,
            "Center should be ~1, got {}",
            center
        );
    }

    #[test]
    fn test_hanning_window_symmetry() {
        let window = create_hanning_window(1024);
        // Should be symmetric
        for i in 0..512 {
            let diff = (window[i] - window[1023 - i]).abs();
            assert!(diff < 1e-6, "Window should be symmetric at {}", i);
        }
    }

    #[test]
    fn test_k_weight_dc_is_zero() {
        // DC (0 Hz) should have zero weight due to high-pass
        let w = k_weight(0.0, 48000.0);
        assert!(w.abs() < 1e-10, "DC should be 0, got {}", w);
    }

    #[test]
    fn test_k_weight_1khz_near_unity() {
        // At 1kHz, K-weighting should be close to unity (0 dB)
        let w = k_weight(1000.0, 48000.0);
        // Allow some tolerance (within 0.5 dB = factor of ~1.06)
        assert!(
            w > 0.9 && w < 1.1,
            "1kHz should be ~1.0 (0dB), got {} ({:.2} dB)",
            w,
            20.0 * w.log10()
        );
    }

    #[test]
    fn test_k_weight_high_shelf_boost() {
        // K-weighting boosts high frequencies (shelving filter)
        // At 4kHz, should have positive gain
        let w = k_weight(4000.0, 48000.0);
        assert!(w > 1.0, "4kHz should have gain > 0dB, got {}", w);
    }

    #[test]
    fn test_k_weight_low_freq_attenuation() {
        // Low frequencies should be attenuated
        let w_100hz = k_weight(100.0, 48000.0);
        let w_1khz = k_weight(1000.0, 48000.0);
        assert!(
            w_100hz < w_1khz,
            "100Hz ({}) should be lower than 1kHz ({})",
            w_100hz,
            w_1khz
        );
    }

    #[test]
    fn test_powers_to_percentages_sum_to_100() {
        let powers = vec![10.0, 20.0, 30.0, 40.0];
        let pct = powers_to_percentages(&powers);
        let sum: f64 = pct.iter().sum();
        assert!(
            (sum - 100.0).abs() < 1e-10,
            "Percentages should sum to 100, got {}",
            sum
        );
    }

    #[test]
    fn test_powers_to_percentages_proportions() {
        let powers = vec![25.0, 25.0, 50.0];
        let pct = powers_to_percentages(&powers);
        assert!((pct[0] - 25.0).abs() < 1e-10);
        assert!((pct[1] - 25.0).abs() < 1e-10);
        assert!((pct[2] - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_powers_to_percentages_zero_total() {
        let powers = vec![0.0, 0.0, 0.0];
        let pct = powers_to_percentages(&powers);
        assert!(pct.iter().all(|&p| p == 0.0));
    }

    #[test]
    fn test_std_dev_constant_values() {
        // Constant values should have std dev = 0
        let values = vec![5.0, 5.0, 5.0, 5.0];
        let sd = std_dev(&values);
        assert!(
            sd.abs() < 1e-10,
            "Constant values should have σ=0, got {}",
            sd
        );
    }

    #[test]
    fn test_std_dev_known_values() {
        // σ of [2, 4, 4, 4, 5, 5, 7, 9] = 2.0
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let sd = std_dev(&values);
        assert!((sd - 2.0).abs() < 1e-10, "Expected σ=2.0, got {}", sd);
    }

    #[test]
    fn test_std_dev_empty() {
        let sd = std_dev(&[]);
        assert!(sd == 0.0);
    }
}
