//! FFT-based audio analysis

use std::sync::Arc;

use rustfft::FftPlanner;
use rustfft::num_complex::Complex;

use super::bands::Band;
use crate::audio::AudioData;

pub(crate) const FFT_SIZE: usize = 16384;
pub(crate) const HOP_SIZE: usize = 2048;

/// Minimum power threshold to avoid log(0) in dB calculations
const MIN_POWER: f64 = 1e-20;

/// Dynamics threshold in dB below band peak (frames below this are considered inaudible)
const DYNAMICS_THRESHOLD_DB: f64 = 60.0;

/// Minimum band power percentage to display dynamics (bands below this show "-")
pub(crate) const DYNAMICS_DISPLAY_THRESHOLD_PCT: f64 = 0.5;

/// Create a Hanning window of the given size
pub(crate) fn create_hanning_window(size: usize) -> Vec<f32> {
    let pi2 = 2.0 * std::f32::consts::PI;
    (0..size)
        .map(|i| 0.5 * (1.0 - (pi2 * i as f32 / (size - 1) as f32).cos()))
        .collect()
}

/// Analyze a single time interval and return band powers
pub(crate) fn analyze_interval(
    samples: &[f32],
    fft: &Arc<dyn rustfft::Fft<f32>>,
    window: &[f32],
    bands: &[Band],
    freq_per_bin: f32,
    k_weights: Option<&[f64]>,
) -> Vec<f64> {
    let nyquist_bin = FFT_SIZE / 2;
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

        pos += HOP_SIZE;
    }

    band_powers
}

/// Convert raw powers to percentages
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

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);

    let mut raw_powers = vec![0.0f64; bands.len()];
    let mut k_powers = vec![0.0f64; bands.len()];
    let mut band_db_per_frame: Vec<Vec<f64>> = vec![Vec::new(); bands.len()];

    let total_frames = if audio.samples.len() >= FFT_SIZE {
        (audio.samples.len() - FFT_SIZE) / HOP_SIZE + 1
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

        pos += HOP_SIZE;
    }

    // Calculate dynamics (standard deviation of dB values)
    // Filter out frames below threshold from band's max (inaudible in normal playback)
    let dynamics: Vec<f64> = band_db_per_frame
        .iter()
        .map(|dbs| {
            if dbs.is_empty() {
                return f64::NEG_INFINITY;
            }

            let max_db = dbs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let threshold = max_db - DYNAMICS_THRESHOLD_DB;

            // Filter frames above threshold and compute stats directly
            let (sum, sum_sq, count) = dbs
                .iter()
                .copied()
                .filter(|&db| db >= threshold)
                .fold((0.0, 0.0, 0usize), |(s, sq, c), db| {
                    (s + db, sq + db * db, c + 1)
                });

            if count == 0 {
                return f64::NEG_INFINITY;
            }

            let n = count as f64;
            let mean = sum / n;
            let variance = (sum_sq / n) - (mean * mean);
            variance.sqrt()
        })
        .collect();

    StatsResult {
        raw_powers,
        k_powers,
        dynamics,
    }
}
