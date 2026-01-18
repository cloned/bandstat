# bandstat

Audio frequency band analyzer with K-weighting and dynamics analysis.

Analyzes the power distribution across frequency bands in audio files, helping identify spectral balance issues in mixes.

## Features

- **Band Power Distribution**: Shows percentage of power in each frequency band (DC to AIR)
- **K-weighting**: ITU-R BS.1770-4 compliant weighting that reflects human hearing perception
- **Dynamics Analysis**: Standard deviation of band power over time (identifies over-compressed frequencies)
- **File Comparison**: Compare up to 5 files side-by-side
- **Timeline Mode**: Track band distribution changes over time

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

### Single File Analysis

```bash
bandstat audio.wav
```

Output includes:
- Raw band power percentages
- K-weighted band power percentages
- Difference between Raw and K-weighted
- Dynamics (standard deviation in dB per band)

### Compare Files

```bash
bandstat file1.wav file2.wav
bandstat original.wav mix_v1.wav mix_v2.wav    # Compare multiple versions
```

First file is used as reference ([A]). Shows differences between files (up to 5).

### Timeline Analysis

```bash
bandstat --time audio.wav
bandstat --time --interval 10 audio.wav      # 10-second intervals
bandstat --time --weighted audio.wav         # Apply K-weighting
```

## Supported Formats

- WAV
- AIFF

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
| AIR  | 18000-22050| Air |

## License

MIT
