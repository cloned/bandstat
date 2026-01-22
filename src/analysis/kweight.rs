//! K-weighting filter implementation (ITU-R BS.1770-4)

use std::f64::consts::PI;

/// Sample rate tolerance for coefficient selection (Hz)
const SAMPLE_RATE_TOLERANCE: f64 = 1.0;

/// K-weighting filter frequency response (ITU-R BS.1770-4)
/// Coefficients:
/// - 48kHz: ITU-R BS.1770-4 Table 1
/// - 44.1kHz: derived via bilinear transform (cf. pyloudnorm, libebur128)
fn k_weight(freq: f64, sample_rate: f64) -> f64 {
    if freq <= 0.0 {
        return 0.0;
    }

    let omega = 2.0 * PI * freq / sample_rate;
    let cos_w = omega.cos();
    let sin_w = omega.sin();
    let cos_2w = (2.0 * omega).cos();
    let sin_2w = (2.0 * omega).sin();

    // Pre-filter (shelving) biquad coefficients from ITU-R BS.1770-4
    let (b0_pre, b1_pre, b2_pre, a1_pre, a2_pre) =
        if (sample_rate - 48000.0).abs() < SAMPLE_RATE_TOLERANCE {
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
    let (b0_rlb, b1_rlb, b2_rlb, a1_rlb, a2_rlb) =
        if (sample_rate - 48000.0).abs() < SAMPLE_RATE_TOLERANCE {
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

/// Create a lookup table of K-weighting factors for FFT bins
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

#[cfg(test)]
pub(super) fn k_weight_for_test(freq: f64, sample_rate: f64) -> f64 {
    k_weight(freq, sample_rate)
}
