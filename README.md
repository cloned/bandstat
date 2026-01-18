# bandstat

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org/)

Audio frequency band analyzer for comparing mixes against reference tracks.

Identifies frequency band differences to help understand your mix's tonal balance. Not intended for precise measurement—for features like LUFS, use ffmpeg which is more reliable.

## Quick Example

Compare your mix with a reference track:

```bash
bandstat your_mix.wav reference.wav -q
```

```
Comparison (base: [A]):
  [A] your_mix.wav
  [B] reference.wav

[Band Power Distribution]
            DC  SUB1  SUB2  BASS  UBAS  LMID   MID  UMID  HMID  PRES  BRIL  HIGH  UHIG   AIR
--------------------------------------------------------------------------------------------
[A]Raw     0.0   0.1   9.1  18.1  21.2  12.1  12.2  10.4  11.6   3.3   1.5   0.3   0.0   0.0
[A]K-wt    0.0   0.0   2.7  10.1  16.4  10.5  11.4  13.7  23.8   7.3   3.3   0.6   0.1   0.0
[A]Diff    0.0  -0.1  -6.4  -8.0  -4.7  -1.7  -0.8  +3.3 +12.2  +4.0  +1.8  +0.4  +0.1   0.0
--------------------------------------------------------------------------------------------
[B]Raw     0.0   0.1   9.2  31.7  15.7   7.0  12.4  10.4   8.4   2.9   2.0   0.2   0.0   0.0
[B]K-wt    0.0   0.1   7.5  25.6  12.6   5.6  10.5  12.5  15.4   5.8   3.9   0.5   0.1   0.0
[B]Diff    0.0   0.0  -1.7  -6.2  -3.1  -1.4  -1.9  +2.1  +7.0  +2.9  +2.0  +0.3   0.0   0.0
--------------------------------------------------------------------------------------------
B-A Raw    0.0   0.0  +0.1 +13.6  -5.5  -5.1  +0.1   0.0  -3.2  -0.4  +0.5   0.0   0.0   0.0
B-A K-wt   0.0  +0.1  +4.8 +15.5  -3.9  -4.9  -0.9  -1.3  -8.4  -1.5  +0.7  -0.1   0.0   0.0
```

**Reading**: B-A Raw shows BASS +13.6%, indicating your mix may lack low-end compared to the reference. Also, your mix's K-wt HMID is 23.8%—this means it sounds quite loud in that range after K-weighting. Check if this is intentional.

## Why bandstat?

Most spectrum analyzers show absolute dB levels per band. Even with gain matching to align overall loudness, you're still comparing absolute values per band—which doesn't directly tell you how the frequency balance is distributed across the spectrum.

bandstat shows **relative power distribution** as percentages. This lets you compare tonal balance directly, regardless of overall loudness. Free, runs from the command line, and outputs numbers instead of graphs.

## Features

- **Band Power Distribution**: Power percentage in 14 frequency bands (DC to AIR)
- **K-weighting**: [ITU-R BS.1770-4](https://www.itu.int/rec/R-REC-BS.1770) weighting that reflects human hearing (optimized for 44.1/48kHz)
- **Dynamics**: Per-band dynamics in dB (lower = more compressed)
- **File Comparison**: Compare up to 10 files
- **Timeline Mode**: Track changes over time

## Installation

Download the latest release from [Releases](../../releases) and extract:

### macOS

```bash
# Apple Silicon
tar xzf bandstat-*-aarch64-apple-darwin.tar.gz

# Intel
tar xzf bandstat-*-x86_64-apple-darwin.tar.gz

./bandstat --help
```

### Linux

```bash
tar xzf bandstat-*-x86_64-unknown-linux-gnu.tar.gz
./bandstat --help
```

### Windows (WSL)

Use the Linux binary via WSL:

```powershell
wsl tar xzf bandstat-*-x86_64-unknown-linux-gnu.tar.gz
wsl ./bandstat audio.wav
```

### Build from Source

Requires Rust 1.85+:

```bash
cargo build --release
./target/release/bandstat --help
```

## Usage

```bash
bandstat audio.wav                              # Single file analysis
bandstat your_mix.wav ref1.wav ref2.wav         # Compare (first file = base)
bandstat --time audio.wav                       # Timeline analysis
bandstat --time --interval 10 -w audio.wav      # Timeline with 10s intervals, K-weighted
bandstat -q audio.wav                           # Quiet mode (suppress explanations)
bandstat --no-color audio.wav                   # Disable colored output
```

### Output

- **Raw(%)**: Power distribution across bands
- **K-wt(%)**: Same as Raw, with K-weighting applied (reflects human hearing)
- **Diff**: K-wt minus Raw (positive = perceived louder than raw measurement)
- **Dyn(dB)**: Per-band dynamics (lower = more compressed). Bands below 0.5% power show "-"

## Supported Formats

- WAV
- AIFF
- MP3
- FLAC

## Frequency Bands

| Band | Range (Hz) | Description |
|------|------------|-------------|
| DC   | 0-20       | Sub-audible |
| SUB1 | 20-40      | Deep sub-bass |
| SUB2 | 40-60      | Sub-bass |
| BASS | 60-120     | Bass |
| UBAS | 120-250    | Upper bass |
| LMID | 250-500    | Low midrange |
| MID  | 500-1000   | Midrange |
| UMID | 1000-2000  | Upper midrange |
| HMID | 2000-4000  | High midrange |
| PRES | 4000-6000  | Presence |
| BRIL | 6000-10000 | Brilliance |
| HIGH | 10000-14000| High frequencies |
| UHIG | 14000-18000| Ultra high |
| AIR  | 18000+     | Air |

**Design notes**: DC band helps detect unwanted DC offset or rumble. Sub-bass is split into SUB1/SUB2 to diagnose low-frequency issues common in home studio mixes (e.g., excessive 40-60Hz buildup vs. true sub-bass).

## License

MIT
