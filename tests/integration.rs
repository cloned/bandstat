//! Integration tests for bandstat CLI

mod common;

use std::process::Command;
use tempfile::TempDir;

/// Get the path to the bandstat binary
fn bandstat_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove deps
    path.push("bandstat");
    path
}

/// Run bandstat with the given arguments
fn run_bandstat(args: &[&str]) -> std::process::Output {
    Command::new(bandstat_bin())
        .args(args)
        .output()
        .expect("failed to execute bandstat")
}

/// Create a test WAV file in the given directory
fn create_test_wav(dir: &TempDir, name: &str, freq: f32, duration: f32) -> std::path::PathBuf {
    let samples = common::generate_sine(freq, 48000, duration);
    let path = dir.path().join(format!("{}.wav", name));
    common::write_wav(&path, &samples, 48000).unwrap();
    path
}

/// Create a noise WAV file in the given directory
fn create_noise_wav(dir: &TempDir, name: &str, duration: f32) -> std::path::PathBuf {
    let samples = common::generate_noise(48000, duration, 12345);
    let path = dir.path().join(format!("{}.wav", name));
    common::write_wav(&path, &samples, 48000).unwrap();
    path
}

// =============================================================================
// Basic functionality tests
// =============================================================================

#[test]
fn test_help_flag() {
    let output = run_bandstat(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Audio frequency band analyzer"));
    assert!(stdout.contains("--time"));
    assert!(stdout.contains("--interval"));
    assert!(stdout.contains("--weighted"));
    assert!(stdout.contains("--image"));
}

#[test]
fn test_version_flag() {
    let output = run_bandstat(&["--version"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bandstat"));
}

// =============================================================================
// Single file analysis mode
// =============================================================================

#[test]
fn test_single_file_analysis() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 1000.0, 2.0);

    let output = run_bandstat(&["-q", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[Band Power Distribution]"));
    assert!(stdout.contains("Raw(%)"));
    assert!(stdout.contains("K-wt(%)"));
    assert!(stdout.contains("[Dynamics]"));
}

#[test]
fn test_single_file_verbose() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 440.0, 1.0);

    let output = run_bandstat(&[wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verbose mode includes file info and bands explanation
    assert!(stdout.contains("Stats Analysis"));
    assert!(stdout.contains("DC"));
    assert!(stdout.contains("AIR"));
}

#[test]
fn test_single_file_with_image() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 1000.0, 2.0);
    let image_path = temp_dir.path().join("output.png");

    let output = run_bandstat(&[
        "-q",
        wav_path.to_str().unwrap(),
        "--image",
        image_path.to_str().unwrap(),
    ]);
    assert!(output.status.success());

    // Check that the image file was created
    assert!(image_path.exists(), "Image file should be created");
    assert!(
        std::fs::metadata(&image_path).unwrap().len() > 0,
        "Image file should not be empty"
    );
}

#[test]
fn test_single_file_with_weighted_image() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 1000.0, 2.0);
    let image_path = temp_dir.path().join("output_k.png");

    let output = run_bandstat(&[
        "-q",
        "-w",
        wav_path.to_str().unwrap(),
        "--image",
        image_path.to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert!(image_path.exists());
}

#[test]
fn test_weighted_without_image_warning() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 1000.0, 1.0);

    let output = run_bandstat(&["-q", "-w", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Warning:"),
        "Should warn about --weighted without --image"
    );
}

// =============================================================================
// Comparison mode (multiple files)
// =============================================================================

#[test]
fn test_compare_two_files() {
    let temp_dir = TempDir::new().unwrap();
    let wav1 = create_test_wav(&temp_dir, "mix", 440.0, 2.0);
    let wav2 = create_test_wav(&temp_dir, "ref", 880.0, 2.0);

    let output = run_bandstat(&["-q", wav1.to_str().unwrap(), wav2.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Comparison (base: [A])"));
    assert!(stdout.contains("[A]"));
    assert!(stdout.contains("[B]"));
    assert!(stdout.contains("B-A"));
}

#[test]
fn test_compare_three_files() {
    let temp_dir = TempDir::new().unwrap();
    let wav1 = create_test_wav(&temp_dir, "a", 440.0, 1.0);
    let wav2 = create_test_wav(&temp_dir, "b", 880.0, 1.0);
    let wav3 = create_test_wav(&temp_dir, "c", 1760.0, 1.0);

    let output = run_bandstat(&[
        "-q",
        wav1.to_str().unwrap(),
        wav2.to_str().unwrap(),
        wav3.to_str().unwrap(),
    ]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[A]"));
    assert!(stdout.contains("[B]"));
    assert!(stdout.contains("[C]"));
    assert!(stdout.contains("B-A"));
    assert!(stdout.contains("C-A"));
}

#[test]
fn test_compare_with_image() {
    let temp_dir = TempDir::new().unwrap();
    let wav1 = create_test_wav(&temp_dir, "a", 440.0, 2.0);
    let wav2 = create_test_wav(&temp_dir, "b", 880.0, 2.0);
    let image_path = temp_dir.path().join("comparison.png");

    let output = run_bandstat(&[
        "-q",
        wav1.to_str().unwrap(),
        wav2.to_str().unwrap(),
        "--image",
        image_path.to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert!(image_path.exists());
}

// =============================================================================
// Timeline mode
// =============================================================================

#[test]
fn test_timeline_mode() {
    let temp_dir = TempDir::new().unwrap();
    // Create a longer file for timeline analysis
    let wav_path = create_noise_wav(&temp_dir, "long", 45.0);

    let output = run_bandstat(&["-q", "-t", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("TIME"));
    assert!(stdout.contains("00:00"));
    assert!(stdout.contains("00:20"));
    assert!(stdout.contains("AVG"));
    assert!(stdout.contains("Duration:"));
}

#[test]
fn test_timeline_with_custom_interval() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_noise_wav(&temp_dir, "long", 25.0);

    let output = run_bandstat(&["-q", "-t", "-i", "10", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("00:00"));
    assert!(stdout.contains("00:10"));
    assert!(stdout.contains("00:20"));
}

#[test]
fn test_timeline_with_k_weighting() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_noise_wav(&temp_dir, "long", 25.0);

    let output = run_bandstat(&["-q", "-t", "-w", wav_path.to_str().unwrap()]);
    assert!(output.status.success());
}

#[test]
fn test_timeline_with_image() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_noise_wav(&temp_dir, "long", 45.0);
    let image_path = temp_dir.path().join("timeline.png");

    let output = run_bandstat(&[
        "-q",
        "-t",
        wav_path.to_str().unwrap(),
        "--image",
        image_path.to_str().unwrap(),
    ]);
    assert!(output.status.success());
    assert!(image_path.exists());
}

// =============================================================================
// Error cases
// =============================================================================

#[test]
fn test_no_files_error() {
    let output = run_bandstat(&[]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("required"));
}

#[test]
fn test_nonexistent_file_error() {
    let output = run_bandstat(&["/nonexistent/path/audio.wav"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error:") || stderr.contains("error"));
}

#[test]
fn test_time_with_multiple_files_error() {
    let temp_dir = TempDir::new().unwrap();
    let wav1 = create_test_wav(&temp_dir, "a", 440.0, 1.0);
    let wav2 = create_test_wav(&temp_dir, "b", 880.0, 1.0);

    let output = run_bandstat(&["-t", wav1.to_str().unwrap(), wav2.to_str().unwrap()]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--time cannot be used with multiple files"));
}

#[test]
fn test_weighted_with_comparison_error() {
    let temp_dir = TempDir::new().unwrap();
    let wav1 = create_test_wav(&temp_dir, "a", 440.0, 1.0);
    let wav2 = create_test_wav(&temp_dir, "b", 880.0, 1.0);

    let output = run_bandstat(&["-w", wav1.to_str().unwrap(), wav2.to_str().unwrap()]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--weighted cannot be used with comparison mode"));
}

#[test]
fn test_interval_without_time_error() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 440.0, 1.0);

    let output = run_bandstat(&["-i", "10", wav_path.to_str().unwrap()]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--interval can only be used with --time"));
}

#[test]
fn test_too_many_files_for_image_error() {
    let temp_dir = TempDir::new().unwrap();
    let wavs: Vec<_> = (0..5)
        .map(|i| create_test_wav(&temp_dir, &format!("f{}", i), 440.0 * (i + 1) as f32, 1.0))
        .collect();
    let image_path = temp_dir.path().join("chart.png");

    let args: Vec<&str> = std::iter::once("-q")
        .chain(wavs.iter().map(|p| p.to_str().unwrap()))
        .chain(["--image", image_path.to_str().unwrap()])
        .collect();

    let output = run_bandstat(&args);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--image supports up to"));
}

#[test]
fn test_image_invalid_directory_error() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 440.0, 1.0);

    let output = run_bandstat(&[
        "-q",
        wav_path.to_str().unwrap(),
        "--image",
        "/nonexistent/dir/chart.png",
    ]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Directory does not exist"));
}

// =============================================================================
// Output format tests
// =============================================================================

#[test]
fn test_no_color_option() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 440.0, 1.0);

    let output = run_bandstat(&["--no-color", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    // Output should not contain ANSI escape codes
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("\x1b["),
        "Should not contain ANSI escape codes"
    );
}

#[test]
fn test_quiet_mode_reduces_output() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_test_wav(&temp_dir, "test", 440.0, 1.0);

    let verbose_output = run_bandstat(&[wav_path.to_str().unwrap()]);
    let quiet_output = run_bandstat(&["-q", wav_path.to_str().unwrap()]);

    let verbose_stdout = String::from_utf8_lossy(&verbose_output.stdout);
    let quiet_stdout = String::from_utf8_lossy(&quiet_output.stdout);

    // Quiet mode should have less output
    assert!(quiet_stdout.len() < verbose_stdout.len());

    // Quiet mode should not include legend
    assert!(!quiet_stdout.contains("Legend:") && !quiet_stdout.contains("Diff:"));
}

// =============================================================================
// Analysis accuracy tests
// =============================================================================

/// Helper to parse Raw(%) or K-wt(%) line values
fn parse_percentage_line(stdout: &str, prefix: &str) -> Option<Vec<f64>> {
    for line in stdout.lines() {
        if line.starts_with(prefix) {
            let values: Vec<f64> = line
                .split_whitespace()
                .skip(1)
                .filter_map(|s| s.parse().ok())
                .collect();
            if !values.is_empty() {
                return Some(values);
            }
        }
    }
    None
}

/// Helper to parse Dyn(dB) line values (may contain "-" for masked values)
fn parse_dynamics_line(stdout: &str) -> Option<Vec<Option<f64>>> {
    for line in stdout.lines() {
        if line.starts_with("Dyn(dB)") {
            let values: Vec<Option<f64>> = line
                .split_whitespace()
                .skip(1)
                .map(|s| if s == "-" { None } else { s.parse().ok() })
                .collect();
            if !values.is_empty() {
                return Some(values);
            }
        }
    }
    None
}

// Band indices (0-indexed):
// 0:DC, 1:SUB1, 2:SUB2, 3:BASS, 4:UBAS, 5:LMID, 6:MID, 7:UMID, 8:HMID,
// 9:PRES, 10:BRIL, 11:HIGH, 12:UHIG, 13:AIR

#[test]
fn test_750hz_sine_in_mid_band() {
    let temp_dir = TempDir::new().unwrap();
    // 750 Hz should be in the MID band (500-1000 Hz)
    let wav_path = create_test_wav(&temp_dir, "750hz", 750.0, 2.0);

    let output = run_bandstat(&["-q", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let values = parse_percentage_line(&stdout, "Raw(%)").expect("Should have Raw(%) line");

    // MID band is index 6
    assert!(
        values[6] > 90.0,
        "750Hz sine should have >90% power in MID band, got {}%",
        values[6]
    );
}

#[test]
fn test_100hz_sine_in_bass_band() {
    let temp_dir = TempDir::new().unwrap();
    // 100 Hz should be in the BASS band (60-120 Hz)
    let wav_path = create_test_wav(&temp_dir, "100hz", 100.0, 2.0);

    let output = run_bandstat(&["-q", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let values = parse_percentage_line(&stdout, "Raw(%)").expect("Should have Raw(%) line");

    // BASS band is index 3
    assert!(
        values[3] > 90.0,
        "100Hz sine should have >90% power in BASS band, got {}%",
        values[3]
    );
}

#[test]
fn test_5khz_sine_in_pres_band() {
    let temp_dir = TempDir::new().unwrap();
    // 5000 Hz should be in the PRES band (4000-6000 Hz)
    let wav_path = create_test_wav(&temp_dir, "5khz", 5000.0, 2.0);

    let output = run_bandstat(&["-q", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let values = parse_percentage_line(&stdout, "Raw(%)").expect("Should have Raw(%) line");

    // PRES band is index 9
    assert!(
        values[9] > 90.0,
        "5kHz sine should have >90% power in PRES band, got {}%",
        values[9]
    );
}

#[test]
fn test_multitone_power_distribution() {
    let temp_dir = TempDir::new().unwrap();
    // Equal amplitude tones at 100Hz (BASS) and 1500Hz (UMID)
    let samples = common::generate_multitone(&[(100.0, 0.5), (1500.0, 0.5)], 48000, 2.0);
    let wav_path = temp_dir.path().join("multitone.wav");
    common::write_wav(&wav_path, &samples, 48000).unwrap();

    let output = run_bandstat(&["-q", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let values = parse_percentage_line(&stdout, "Raw(%)").expect("Should have Raw(%) line");

    // Both BASS (index 3) and UMID (index 7) should have significant power
    // Since equal amplitude, they should be roughly equal (within 20% of each other)
    let bass = values[3];
    let umid = values[7];
    assert!(
        bass > 30.0 && umid > 30.0,
        "Both bands should have >30% power: BASS={}%, UMID={}%",
        bass,
        umid
    );
    assert!(
        (bass - umid).abs() < 20.0,
        "Power should be roughly equal: BASS={}%, UMID={}%",
        bass,
        umid
    );
}

#[test]
fn test_percentages_sum_to_100() {
    let temp_dir = TempDir::new().unwrap();
    let wav_path = create_noise_wav(&temp_dir, "noise", 2.0);

    let output = run_bandstat(&["-q", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let raw_values = parse_percentage_line(&stdout, "Raw(%)").expect("Should have Raw(%) line");
    let kwt_values = parse_percentage_line(&stdout, "K-wt(%)").expect("Should have K-wt(%) line");

    let raw_sum: f64 = raw_values.iter().sum();
    let kwt_sum: f64 = kwt_values.iter().sum();

    // Allow 0.2% tolerance due to rounding in display
    assert!(
        (raw_sum - 100.0).abs() < 0.2,
        "Raw percentages should sum to 100%, got {}%",
        raw_sum
    );
    assert!(
        (kwt_sum - 100.0).abs() < 0.2,
        "K-wt percentages should sum to 100%, got {}%",
        kwt_sum
    );
}

#[test]
fn test_k_weighting_reduces_low_freq() {
    let temp_dir = TempDir::new().unwrap();
    // Mix of 100 Hz and 2000 Hz - K-weighting should shift balance
    let samples = common::generate_multitone(&[(100.0, 0.5), (2000.0, 0.5)], 48000, 2.0);
    let wav_path = temp_dir.path().join("low_high_mix.wav");
    common::write_wav(&wav_path, &samples, 48000).unwrap();

    let output = run_bandstat(&["-q", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let raw_values = parse_percentage_line(&stdout, "Raw(%)").expect("Should have Raw(%) line");
    let kwt_values = parse_percentage_line(&stdout, "K-wt(%)").expect("Should have K-wt(%) line");

    // BASS band (index 3) should have lower K-wt% than Raw%
    // K-weighting attenuates low frequencies relative to mid/high
    assert!(
        kwt_values[3] < raw_values[3],
        "K-weighting should reduce BASS power: Raw={}%, K-wt={}%",
        raw_values[3],
        kwt_values[3]
    );

    // UMID band (index 7) should have higher K-wt% than Raw%
    assert!(
        kwt_values[7] > raw_values[7],
        "K-weighting should boost UMID power: Raw={}%, K-wt={}%",
        raw_values[7],
        kwt_values[7]
    );
}

#[test]
fn test_k_weighting_boosts_presence() {
    let temp_dir = TempDir::new().unwrap();
    // Broadband noise - K-weighting should shift balance toward higher frequencies
    let wav_path = create_noise_wav(&temp_dir, "noise", 2.0);

    let output = run_bandstat(&["-q", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let raw_values = parse_percentage_line(&stdout, "Raw(%)").expect("Should have Raw(%) line");
    let kwt_values = parse_percentage_line(&stdout, "K-wt(%)").expect("Should have K-wt(%) line");

    // Sum of low bands (DC through UBAS, indices 0-4)
    let raw_low: f64 = raw_values[0..5].iter().sum();
    let kwt_low: f64 = kwt_values[0..5].iter().sum();

    // K-weighting should reduce low frequency content
    assert!(
        kwt_low < raw_low,
        "K-weighting should reduce low freq sum: Raw={}%, K-wt={}%",
        raw_low,
        kwt_low
    );
}

#[test]
fn test_dynamics_constant_amplitude_low() {
    let temp_dir = TempDir::new().unwrap();
    // Constant amplitude sine wave should have very low dynamics
    let wav_path = create_test_wav(&temp_dir, "constant", 1000.0, 3.0);

    let output = run_bandstat(&["-q", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let dyn_values = parse_dynamics_line(&stdout).expect("Should have Dyn(dB) line");

    // MID band (index 6) should have low dynamics for constant amplitude
    if let Some(mid_dyn) = dyn_values[6] {
        assert!(
            mid_dyn < 1.0,
            "Constant amplitude should have dynamics < 1.0 dB, got {} dB",
            mid_dyn
        );
    }
}

#[test]
fn test_dynamics_varying_amplitude_higher() {
    let temp_dir = TempDir::new().unwrap();
    // Create a signal with varying amplitude (fade in/out pattern)
    let samples = common::generate_sine_with_envelope(1000.0, 48000, 4.0, |t| {
        // Pulsing envelope: amplitude varies from 0.2 to 1.0 several times
        let cycles = 4.0;
        0.2 + 0.8 * ((t * cycles * 2.0 * std::f32::consts::PI).sin() * 0.5 + 0.5)
    });
    let wav_path = temp_dir.path().join("varying.wav");
    common::write_wav(&wav_path, &samples, 48000).unwrap();

    let output = run_bandstat(&["-q", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let dyn_values = parse_dynamics_line(&stdout).expect("Should have Dyn(dB) line");

    // MID band (index 6) should have higher dynamics for varying amplitude
    if let Some(mid_dyn) = dyn_values[6] {
        assert!(
            mid_dyn > 1.0,
            "Varying amplitude should have dynamics > 1.0 dB, got {} dB",
            mid_dyn
        );
    }
}

#[test]
fn test_comparison_diff_calculation() {
    let temp_dir = TempDir::new().unwrap();
    // File A: 100Hz (BASS dominant, index 3)
    // File B: 750Hz (MID dominant, index 6)
    let wav_a = create_test_wav(&temp_dir, "bass", 100.0, 2.0);
    let wav_b = create_test_wav(&temp_dir, "mid", 750.0, 2.0);

    let output = run_bandstat(&["-q", wav_a.to_str().unwrap(), wav_b.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Find B-A Raw line and check that BASS decreased, MID increased
    let mut found_diff = false;
    for line in stdout.lines() {
        if line.contains("B-A") && line.contains("Raw") {
            found_diff = true;
            let values: Vec<f64> = line
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();

            if values.len() >= 7 {
                // BASS (index 3) should be very negative (B has ~0% bass, A has ~100%)
                assert!(
                    values[3] < -80.0,
                    "B-A BASS diff should be < -80: got {}",
                    values[3]
                );
                // MID (index 6) should be very positive (B has ~100% mid, A has ~0%)
                assert!(
                    values[6] > 80.0,
                    "B-A MID diff should be > 80: got {}",
                    values[6]
                );
            }
            break;
        }
    }
    assert!(found_diff, "Should find B-A Raw diff line");
}

#[test]
fn test_timeline_tracks_frequency_change() {
    let temp_dir = TempDir::new().unwrap();
    // Create audio that changes frequency halfway through
    // First half: 100Hz, Second half: 3000Hz
    let sr = 48000u32;
    let duration = 10.0f32; // 10 seconds total
    let half_samples = (sr as f32 * duration / 2.0) as usize;

    let mut samples = Vec::with_capacity((sr as f32 * duration) as usize);

    // First half: 100Hz (BASS band)
    for i in 0..half_samples {
        let t = i as f32 / sr as f32;
        samples.push((2.0 * std::f32::consts::PI * 100.0 * t).sin() * 0.5);
    }
    // Second half: 3000Hz (HMID band)
    for i in 0..half_samples {
        let t = i as f32 / sr as f32;
        samples.push((2.0 * std::f32::consts::PI * 3000.0 * t).sin() * 0.5);
    }

    let wav_path = temp_dir.path().join("freq_change.wav");
    common::write_wav(&wav_path, &samples, sr).unwrap();

    // Use 5 second intervals
    let output = run_bandstat(&["-q", "-t", "-i", "5", wav_path.to_str().unwrap()]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have 00:00 (bass) and 00:05 (hmid) intervals
    let mut found_00_00 = false;
    let mut found_00_05 = false;

    for line in stdout.lines() {
        if line.starts_with("00:00") {
            found_00_00 = true;
            // First interval should have BASS (index 3) dominant
            let values: Vec<f64> = line
                .split_whitespace()
                .skip(1)
                .filter_map(|s| s.parse().ok())
                .collect();
            if values.len() >= 4 {
                assert!(
                    values[3] > 80.0,
                    "First interval should be BASS dominant: {}%",
                    values[3]
                );
            }
        }
        if line.starts_with("00:05") {
            found_00_05 = true;
            // Second interval should have HMID (index 8) dominant
            let values: Vec<f64> = line
                .split_whitespace()
                .skip(1)
                .filter_map(|s| s.parse().ok())
                .collect();
            if values.len() >= 9 {
                assert!(
                    values[8] > 80.0,
                    "Second interval should be HMID dominant: {}%",
                    values[8]
                );
            }
        }
    }

    assert!(found_00_00, "Should have 00:00 interval");
    assert!(found_00_05, "Should have 00:05 interval");
}
