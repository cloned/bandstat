# bandstat

bandstat is a command-line audio frequency band analyzer for comparing mixes against reference tracks.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org/)

[日本語](README.ja.md)

## Example

Compare your mix with a reference track:

```
$ bandstat -q your_mix.wav reference.wav
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

**How to read**: B-A Raw shows BASS +13.6%, meaning the reference has 13.6% more bass than your mix. Your mix's K-wt HMID is 23.8%, indicating that range sounds perceptually prominent.

Track how frequency balance changes across sections:

```
$ bandstat -tq audio.wav
TIME      DC  SUB1  SUB2  BASS  UBAS  LMID   MID  UMID  HMID  PRES  BRIL  HIGH  UHIG   AIR
------------------------------------------------------------------------------------------
00:00    0.0   0.1   4.5  11.2  15.7  21.8  22.0  15.1   6.1   2.1   1.1   0.3   0.1   0.0
00:20    0.0   0.1   9.4  15.6  27.0   9.2   9.7   9.0  14.3   3.7   1.6   0.4   0.1   0.0
00:40    0.0   0.1  12.6  23.8  21.3   8.4   7.4   7.7  13.0   3.6   1.7   0.3   0.0   0.0
01:00    0.0   0.1  10.1  24.6  18.3   7.5   9.0  10.1  14.2   4.6   1.4   0.2   0.0   0.0
------------------------------------------------------------------------------------------
AVG      0.0   0.1   9.1  18.0  21.1  12.2  12.3  10.4  11.6   3.3   1.5   0.3   0.0   0.0
```

Useful when the track's overall average doesn't reflect section-by-section differences—for example, an intro with sparse mids vs. a bass-heavy chorus.

## Why bandstat?

When A/B-ing your mix against a reference track, you can hear that something's different, but pinpointing which frequencies differ and by how much is hard. Real-time spectrum analyzers don't make this easier—reading differences from moving waveforms is impractical, and you end up with vague impressions like "maybe the low end is lacking."

bandstat takes two audio files and outputs the power distribution of each frequency band as percentages, along with the difference. You get concrete numbers like "BASS is 13% lower, HMID is 8% higher." Since it uses percentages, you can compare files with different LUFS levels directly without gain matching.

## Features

* 14 frequency bands from DC to AIR
* K-weighting based on [ITU-R BS.1770-4](https://www.itu.int/rec/R-REC-BS.1770) (optimized for 44.1/48kHz)
* Per-band dynamics analysis
* Compare up to 10 files
* Timeline mode for tracking changes over time

## Installation

[Releases](../../releases) has precompiled binaries for the following platforms:

| Platform | Archive |
|----------|---------|
| **macOS (Apple Silicon)** | `bandstat-vX.X.X-aarch64-apple-darwin.tar.gz` |
| **macOS (Intel)** | `bandstat-vX.X.X-x86_64-apple-darwin.tar.gz` |
| **Linux (x86_64)** | `bandstat-vX.X.X-x86_64-unknown-linux-gnu.tar.gz` |

Download and extract:

```
$ tar xzf bandstat-*.tar.gz
$ cd bandstat-*/
$ ./bandstat --help
```

If you're a **Windows** user, use the Linux binary via WSL.

## Usage

```
bandstat audio.wav                                   # Single file analysis
bandstat my_mix.wav ref.wav                          # Compare files (first = base)
bandstat --time audio.wav                            # Timeline analysis
bandstat --time --interval 10 --weighted audio.wav   # 10s intervals, K-weighted
bandstat --quiet audio.wav                           # Quiet mode
bandstat --no-color audio.wav                        # Disable colored output
```

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--time` | `-t` | Timeline analysis mode |
| `--interval <SECONDS>` | `-i` | Timeline interval (default: 20) |
| `--weighted` | `-w` | Apply K-weighting to timeline |
| `--quiet` | `-q` | Suppress explanations |
| `--no-color` | | Disable colored output |

### Output columns

* **Raw(%)**: Power distribution across bands
* **K-wt(%)**: Same as Raw, with K-weighting applied
* **Diff**: K-wt minus Raw (positive = perceived louder than measured)
* **Dyn(dB)**: Per-band dynamics (lower = more compressed)

### Supported formats

WAV, AIFF, MP3, FLAC

### Frequency bands

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

DC band helps detect unwanted DC offset or rumble. Sub-bass is split into SUB1/SUB2 to diagnose low-frequency issues that can be hard to distinguish depending on your monitoring environment.

## Building

bandstat is written in Rust. Building requires Rust 1.85 or newer.

```
$ git clone https://github.com/cloned/bandstat.git
$ cd bandstat
$ cargo build --release
$ ./target/release/bandstat --help
```

### Running tests

```
$ cargo test
```

## License

MIT
