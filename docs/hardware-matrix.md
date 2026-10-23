# Hardware and compatibility

Dubbit runs on the CPU everywhere. That is the guaranteed path. Nothing in this
page promises graphics acceleration: this build does not use a graphics adapter
for inference, so the matrix below only records what has actually run.

## States

Every row uses one of these words, and nothing else:

- **Verified, author machine**: the full pipeline ran to completion on that machine.
- **Verified, continuous integration**: the checks and tests ran on that platform in CI.
- **Implemented, not verified**: the code path exists but has not run on that platform.
- **Not supported**: it does not work and is not planned for the current milestone.

## Operating systems

| System | Architecture | State | Notes |
| --- | --- | --- | --- |
| Windows 11 | x86_64 | Verified, author machine | Full pipeline, model download, diagnostics |
| Linux | x86_64 | Implemented, not verified | Same code path; CI builds and tests it |
| macOS | Apple Silicon | Implemented, not verified | Toolchain builds; no end-to-end run yet |
| macOS | Intel | Not supported | No test machine and no verified build |

## Profiles

| Profile | Target hardware | Model | State |
| --- | --- | --- | --- |
| light | 8 GB of RAM, no useful graphics adapter | `whisper-tiny-q5_1` | Verified, author machine |
| balanced | 16 GB of RAM or a recent Apple Silicon Mac | `whisper-small-q5_1` | Implemented, not verified |
| max | 32 GB of RAM or a discrete graphics adapter with 12 GB | `whisper-large-v3-turbo-q5_0` | Implemented, not verified |
| auto | Detected from memory, cores and adapters | Chosen per machine | Verified, author machine |

The profile decides the model and the thread count. It never blocks a run: if
the chosen model is slower than expected, the CPU path still finishes.

## Graphics adapters

| Adapter family | Acceleration | State |
| --- | --- | --- |
| NVIDIA | None in this build | Implemented as detection only |
| AMD | None in this build | Detected, not accelerated |
| Intel | None in this build | Detected, not accelerated |
| Apple Silicon | None in this build | Detected, not accelerated |

`dubbit doctor` prints the adapters it finds and explains which profile it
chose. Detection is information, not a claim of acceleration.

## Media

| Input | State | Notes |
| --- | --- | --- |
| MP4 with H.264 and AAC | Verified, author machine | Audio extracted to 16 kHz mono WAV |
| WAV, 16-bit PCM | Verified, author machine | Used directly |
| MKV, MOV, WebM | Implemented, not verified | Accepted when FFmpeg reads them |
| Audio-only files | Implemented, not verified | Same pipeline without video |
| Paths with accents, spaces and non-ASCII characters | Implemented, not verified | Unicode paths are used throughout |
| Damaged or truncated files | Rejected with an error code | `DUB-1002` or `DUB-1004` |

## Languages

All 65 codes listed by `dubbit languages` can be transcribed and used as a
target. Quality differs by language and model, and no language has a measured
quality grade yet. Treat every pair as unproven until the benchmark work in the
roadmap is done.

## What would change a row

A row moves to verified only after the full pipeline finishes on that machine,
with the command and the output recorded in the pull request. Detection, a
successful build, or a passing unit test is not enough.
