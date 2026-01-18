use std::f32::consts::PI;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn write_wav(path: &Path, samples: &[f32], sample_rate: u32) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
    let block_align = channels * bits_per_sample / 8;
    let data_size = samples.len() as u32 * 2;
    let file_size = 36 + data_size;

    writer.write_all(b"RIFF")?;
    writer.write_all(&file_size.to_le_bytes())?;
    writer.write_all(b"WAVE")?;
    writer.write_all(b"fmt ")?;
    writer.write_all(&16u32.to_le_bytes())?;
    writer.write_all(&1u16.to_le_bytes())?;
    writer.write_all(&channels.to_le_bytes())?;
    writer.write_all(&sample_rate.to_le_bytes())?;
    writer.write_all(&byte_rate.to_le_bytes())?;
    writer.write_all(&block_align.to_le_bytes())?;
    writer.write_all(&bits_per_sample.to_le_bytes())?;
    writer.write_all(b"data")?;
    writer.write_all(&data_size.to_le_bytes())?;

    for &sample in samples {
        let value = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_all(&value.to_le_bytes())?;
    }

    Ok(())
}

fn sine(freq: f32, duration: f32, sample_rate: u32) -> Vec<f32> {
    let n = (duration * sample_rate as f32) as usize;
    (0..n)
        .map(|i| 0.1 * (2.0 * PI * freq * i as f32 / sample_rate as f32).sin())
        .collect()
}

fn mix(a: &[f32], b: &[f32]) -> Vec<f32> {
    a.iter().zip(b).map(|(x, y)| x + y).collect()
}

fn allband(duration: f32, sample_rate: u32) -> Vec<f32> {
    // One sine wave per band (center frequency, skipping DC and AIR)
    let freqs = [
        30.0,    // SUB1: 20-40
        50.0,    // SUB2: 40-60
        90.0,    // BASS: 60-120
        180.0,   // UBAS: 120-250
        375.0,   // LMID: 250-500
        750.0,   // MID: 500-1000
        1500.0,  // UMID: 1000-2000
        3000.0,  // HMID: 2000-4000
        5000.0,  // PRES: 4000-6000
        8000.0,  // BRIL: 6000-10000
        12000.0, // HIGH: 10000-14000
        16000.0, // UHIG: 14000-18000
        19000.0, // AIR: 18000+
    ];

    let n = (duration * sample_rate as f32) as usize;
    let amp = 0.05; // Lower amplitude since we're summing many waves

    (0..n)
        .map(|i| {
            freqs
                .iter()
                .map(|&f| amp * (2.0 * PI * f * i as f32 / sample_rate as f32).sin())
                .sum()
        })
        .collect()
}

fn main() -> std::io::Result<()> {
    let sr = 44100;
    let dur = 3.0;
    let dir = Path::new("test_data");
    std::fs::create_dir_all(dir)?;

    // 1kHz sine wave (UMID band: 1000-2000Hz)
    write_wav(&dir.join("1khz.wav"), &sine(1000.0, dur, sr), sr)?;

    // 100Hz sine wave (BASS band: 60-120Hz)
    write_wav(&dir.join("100hz.wav"), &sine(100.0, dur, sr), sr)?;

    // 5kHz sine wave (PRES band: 4000-6000Hz)
    write_wav(&dir.join("5khz.wav"), &sine(5000.0, dur, sr), sr)?;

    // 100Hz + 3kHz mix (for K-weighting test)
    // Raw: ~50% each, K-wt: 3kHz boosted, 100Hz attenuated
    let low = sine(100.0, dur, sr);
    let high = sine(3000.0, dur, sr);
    write_wav(&dir.join("mix_100_3000hz.wav"), &mix(&low, &high), sr)?;

    // All-band test: one sine per band (~7.7% each in Raw)
    write_wav(&dir.join("allband.wav"), &allband(dur, sr), sr)?;

    println!("Generated: 1khz.wav, 100hz.wav, 5khz.wav, mix_100_3000hz.wav, allband.wav");
    Ok(())
}
