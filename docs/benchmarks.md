# Benchmarks

Dubbit publishes only measurements it made itself, with the command used to make
them. Two measurements exist: a short clip that shows start-up cost, and a long
file that shows throughput under a hard memory cap.

## How to reproduce a transcription measurement

```sh
cargo build --release
./target/release/dubbit models install whisper-tiny-q5_1
time ./target/release/dubbit dub samples/clip.wav --from en --no-translate
```

`--no-translate` isolates transcription. Report the media duration, the wall
time, the model, the profile, the operating system, the processor, the memory
limit and the peak memory.

On Windows the measurement can be repeated inside a job object with a memory
cap, which is how the numbers below were produced. The wall time covers the
whole command: process start, audio extraction, transcription and subtitle
writing.

## Measurement 1: short clip

| Field | Value |
| --- | --- |
| Date | 2026-10-23 |
| Media | 7.36 s of clean English speech, 16-bit PCM WAV, mono |
| Model | `whisper-tiny-q5_1` |
| Profile | light, auto-selected |
| Build | release |
| System | Windows 11, 12 logical cores, 15 GB of RAM |
| Graphics | Discrete adapter detected, not used for inference |
| Wall time | 1.86 s, 1.70 s and 1.70 s over three runs |
| Ratio | about 4x faster than real time |

At this length the run is dominated by process start and model load, so the
ratio understates throughput. Measurement 2 is the useful one.

## Measurement 2: long file under a hard 8 GiB cap

| Field | Value |
| --- | --- |
| Date | 2026-10-23 |
| Media | 817.26 s (13 min 37 s) of continuous synthesised English speech, WAV, 36 MB, mono |
| System | Windows 11, 12 logical cores, 15 GB of RAM |
| Graphics | Discrete adapter detected, not used for inference |
| Build | release |
| Memory limit | 8 GiB, enforced by a Windows job object on the process and the job, including the recognition child process |
| Network | no request is made in the processing code path; not instrumented at the socket level in this run |
| Translation | disabled |

| Model | Profile | Wall time | Ratio | Peak job memory |
| --- | --- | --- | --- | --- |
| `whisper-tiny-q5_1` | light | 47.64 s | 17.2x real time | 731.5 MB |
| `whisper-base-q5_1` | light, model forced | 69.79 s | 11.7x real time | 789.2 MB |
| `whisper-small-q5_1` | balanced | 200.44 s | 4.1x real time | 1055.0 MB |

All three runs completed with exit code 0 and produced 223 to 224 subtitle cues
covering the whole file, from the first sentence to the last.

The cap never bound: peak use stayed between 0.7 GB and 1.1 GB. That establishes
that these models fit comfortably inside an 8 GB machine with the operating
system and a browser running, but it does not exercise an out-of-memory path.

### What this measurement does not say

- The speech was synthesised from one voice, with no music, noise, crosstalk or
  accent variation. Real media is harder for the recogniser and for timing.
- Accuracy was not measured. There is no word error rate yet.
- One machine, one operating system. Linux, macOS and smaller processors are
  unmeasured.
- Ratio falls quickly with model size. A 13 minute file on the small model still
  takes over three minutes, so long recordings need the light or base model on
  modest hardware.

## Planned measurements

The roadmap calls for a wider suite: several clip lengths, noisy and
multi-speaker audio, all four models, every profile, and a word error rate
against a public test set. Results will be added here with the raw numbers and
the exact commands, and previous entries will be kept rather than edited.

## Rules

- No number appears here without a command that produced it.
- A run that failed is reported as a failure, not removed.
- No comparison with another product appears in a benchmark table.
