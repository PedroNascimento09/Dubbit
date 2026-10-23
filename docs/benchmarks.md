# Benchmarks

Dubbit publishes only measurements it made itself, with the command used to make
them. There is one measurement so far. It is small, and it is labelled as such.

## How to reproduce a transcription measurement

```sh
cargo build --release
./target/release/dubbit models install whisper-tiny-q5_1
time ./target/release/dubbit dub samples/clip.wav --from en --no-translate
```

`--no-translate` isolates transcription. `--output` puts the project folder
somewhere you can delete afterwards. Report the media duration, the wall time,
the model, the profile, the operating system, the processor and the memory.

## Measurement 1

| Field | Value |
| --- | --- |
| Date | 2026-10-23 |
| Media | 7.36 s of clean English speech, 16-bit PCM WAV, mono |
| Model | `whisper-tiny-q5_1` |
| Profile | light, auto-selected |
| System | Windows 11, 12 logical cores, 15 GB of RAM |
| Graphics | Discrete adapter detected, not used for inference |
| Build | release, `cargo build --release` |
| Wall time | 1.86 s, 1.70 s and 1.70 s over three runs, whole command, including audio extraction |
| Ratio | about 4x faster than real time |

### What this measurement does not say

- It does not predict longer files. Fixed costs and thermal limits change the ratio.
- It does not compare models, profiles or hardware. One sample cannot.
- It does not measure translation, which was disabled.
- It does not measure accuracy. Word error rate has not been measured yet.

## Planned measurements

The roadmap calls for a repeatable benchmark suite: several clip lengths, noisy
and multi-speaker audio, all four models, each profile, and a word error rate
against a public test set. Results will be added here with the raw numbers and
the exact commands, and previous entries will be kept rather than edited.

## Rules

- No number appears here without a command that produced it.
- A run that failed is reported as a failure, not removed.
- No comparison with another product appears in a benchmark table.
