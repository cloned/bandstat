//! Unit tests for analysis module

use super::fft::{create_hanning_window, powers_to_percentages};
use super::kweight::k_weight_for_test;

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
    let w = k_weight_for_test(0.0, 48000.0);
    assert!(w.abs() < 1e-10, "DC should be 0, got {}", w);
}

#[test]
fn test_k_weight_1khz_near_unity() {
    // At 1kHz, K-weighting should be close to unity (0 dB)
    let w = k_weight_for_test(1000.0, 48000.0);
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
    let w = k_weight_for_test(4000.0, 48000.0);
    assert!(w > 1.0, "4kHz should have gain > 0dB, got {}", w);
}

#[test]
fn test_k_weight_low_freq_attenuation() {
    // Low frequencies should be attenuated
    let w_100hz = k_weight_for_test(100.0, 48000.0);
    let w_1khz = k_weight_for_test(1000.0, 48000.0);
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
