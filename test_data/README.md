# Test Data

Test WAV files for verifying bandstat output.

## Files

| File | Content | Expected |
|------|---------|----------|
| 100hz.wav | 100 Hz | ~100% in BASS |
| 1khz.wav | 1000 Hz | 100% in UMID |
| 5khz.wav | 5000 Hz | 100% in PRES |
| mix_100_3000hz.wav | 100 Hz + 3000 Hz | Raw: ~50% each, K-wt: HMID > BASS |

## Verify

```
./verify.sh
```

Tests:
1. Band classification - pure tones appear in correct bands
2. K-weighting - 1kHz unchanged, low/high mix shows attenuation/boost
