# Test Data

Test WAV files for verifying bandstat output.

## Files

| File | Content | Expected |
|------|---------|----------|
| 100hz.wav | 100 Hz sine | ~100% in BASS |
| 1khz.wav | 1000 Hz sine | 100% in UMID |
| 5khz.wav | 5000 Hz sine | 100% in PRES |
| mix_100_3000hz.wav | 100 Hz + 3000 Hz | Raw: ~50% each, K-wt: HMID > BASS |
| allband.wav | 13 sines (one per band) | ~7.7% each in Raw, K-wt shifts to high |
| 1khz.aiff | 1kHz sine (AIFF) | 100% in UMID (tests AIFF format) |
| 1khz.mp3 | 1kHz sine (MP3) | ~100% in UMID (tests MP3 format) |
| 1khz.flac | 1kHz sine (FLAC) | 100% in UMID (tests FLAC format) |
| 1khz_96k.wav | 1kHz sine @ 96kHz | 100% in UMID + K-weight warning |

## Verify

```
./verify.sh
```

Tests:
1. Band classification - pure tones appear in correct bands
2. K-weighting - 1kHz unchanged, low/high mix shows attenuation/boost
3. All-band distribution - uniform distribution, K-weighting effect across all bands
