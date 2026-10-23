# Dubbit

**Subtitles and translations for your videos, on your computer.**

Dubbit reads a video or audio file, transcribes the speech, translates it, and writes subtitle files. Everything runs on your machine: no accounts, no uploads, no telemetry, and no network while it works.

This is an early build. It is a command line tool, built from source, and its promise today is subtitles only. Dubbing is on the roadmap, not in the build.

## What works today

- Transcribe speech with word-accurate timing and write SRT and VTT subtitles.
- Translate subtitles into any of the 65 supported language codes, or keep the original language only.
- Resume an interrupted run: finished stages are skipped and the source file is checksum-checked before continuing.
- Choose a hardware profile, or let Dubbit pick one from the machine it finds.
- Download models with resumable transfers and SHA-256 verification, or manage them by hand.
- Keep terminology consistent with a CSV glossary, protected through translation.
- Read errors as a code, a plain title, a reason and a next step.
- Export a diagnostic report with one command.

## What is not here yet

- No installers and no desktop window. You build the binary with Cargo.
- No dubbing, voice synthesis or lip sync.
- macOS has not been tested. Windows and Linux are the supported targets.
- FFmpeg and the speech recognition executable are supplied by you, not bundled yet.
- Translation quality has not been measured. Treat it as unproven until the benchmark work lands.

## Quick start

You need Rust (the pinned toolchain in `rust-toolchain.toml`), FFmpeg and FFprobe, and a whisper.cpp `whisper-cli` build. See [docs/install.md](docs/install.md) for the details on each system.

```sh
cargo build --release
./target/release/dubbit doctor
./target/release/dubbit models install whisper-tiny-q5_1
./target/release/dubbit dub interview.mp4 --to pt
```

The last command writes a project folder next to the input, `interview.dubbit`, with:

```text
interview.dubbit/
  project.json
  audio.wav
  segments-source.json
  segments-translated.json
  subtitles/en.srt
  subtitles/en.vtt
  subtitles/pt.srt
  subtitles/pt.vtt
```

## Commands

| Command | What it does |
| --- | --- |
| `dubbit dub FILE` | Transcribe, translate and write subtitles |
| `dubbit models list` | Show models with size, memory estimate, licence and state |
| `dubbit models install ID` | Download and verify a model, resuming a partial file |
| `dubbit models remove ID` | Delete a downloaded model |
| `dubbit models verify` | Re-check every installed model against the manifest |
| `dubbit doctor` | Print the machine, tools, profile and models |
| `dubbit doctor --report FILE` | Write the same report to a file for an issue |
| `dubbit languages` | List supported language codes |
| `dubbit config show` / `set` / `path` | Read or change settings |
| `dubbit project DIR` | Show the stages, options and files of a project |

Useful flags on `dub`:

- `--to LANG` target language, `--from LANG` source language when detection is wrong.
- `--no-translate` for subtitles in the original language only.
- `--translator auto|passthrough|ollama|command` to choose the translation backend.
- `--model ID`, `--profile auto|light|balanced|max`, `--threads N`.
- `--format srt,vtt`, `--glossary terms.csv`, `--output DIR`, `--resume`.
- `--json` on any command for machine readable output.

Full usage, settings and examples live in [docs/user-guide.md](docs/user-guide.md).

## Models

| Model | Size on disk | Memory estimate | Profile |
| --- | --- | --- | --- |
| `whisper-tiny-q5_1` | 30.7 MB | 0.4 GB | light |
| `whisper-base-q5_1` | 56.9 MB | 0.7 GB | manual |
| `whisper-small-q5_1` | 181.3 MB | 1.5 GB | balanced |
| `whisper-large-v3-turbo-q5_0` | 547.4 MB | 4 GB | max |

All four entries are pinned by URL, byte size and SHA-256 in [models/whisper.json](models/whisper.json), and all are MIT licensed. Dubbit never ships weights in the repository and never downloads anything you did not ask for. Licences and sources are listed in [MODEL_LICENSES.md](MODEL_LICENSES.md).

## Profiles

| Profile | Model | RAM budget |
| --- | --- | --- |
| light | `whisper-tiny-q5_1` | 4 GB |
| balanced | `whisper-small-q5_1` | 8 GB |
| max | `whisper-large-v3-turbo-q5_0` | 16 GB |

`auto` picks from detected memory, cores and graphics adapters. The CPU path is the guaranteed path: a graphics adapter never blocks a run.

## Performance

One measurement, made by the author on Windows 11, 12 cores, 15 GB of RAM, no usable graphics acceleration, with `whisper-tiny-q5_1`:

| Audio | Wall time | Ratio |
| --- | --- | --- |
| 7.36 s of English speech | 1.70 s to 1.86 s over three runs | about 4x faster than real time |

That is a single short sample, measured with a release build, including the FFmpeg process and its start-up cost. It is not a benchmark suite and not a promise for longer media. Reproducible benchmark scripts and a wider hardware matrix are on the roadmap. See [docs/benchmarks.md](docs/benchmarks.md).

## Privacy

Processing is local. The only outbound request is a model download you start yourself. Diagnostic reports are written where you ask and never sent automatically. Details in [docs/PRIVACY.md](docs/PRIVACY.md).

## Responsible use

Translate and subtitle media you have the right to use. See [docs/ACCEPTABLE_USE.md](docs/ACCEPTABLE_USE.md). Voice cloning is not part of this build, and any future voice feature will be off by default with a consent step.

## Errors you may meet

Every failure carries a stable code such as `DUB-1001` (a media tool is missing) or `DUB-3104` (the model is not installed), a human title, a reason and a next step. The full catalogue and fixes are in [docs/troubleshooting.md](docs/troubleshooting.md).

## Documentation

- [Install](docs/install.md)
- [User guide](docs/user-guide.md)
- [Troubleshooting](docs/troubleshooting.md)
- [Hardware and compatibility](docs/hardware-matrix.md)
- [Benchmarks](docs/benchmarks.md)
- [Development](docs/DEVELOPMENT.md)
- [Roadmap](ROADMAP.md)
- [Changelog](CHANGELOG.md)
- [Architecture decisions](docs/adr/)
- [Licence audit](docs/LICENSES-AUDIT.md)

## Contributing

Issues and pull requests are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) first: the checks are `cargo fmt`, `cargo clippy` with warnings as errors, and `cargo test`, and every commit carries a DCO sign-off. Good first issues are labelled in the tracker.

## Licence, trademark and official channels

Dubbit is licensed under Apache-2.0. See [LICENSE](LICENSE), [NOTICE](NOTICE), [TRADEMARKS.md](TRADEMARKS.md) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). The licence covers the code, not the name, the logo or the mascot.

The only official channels are this repository, its releases and its discussions. Download releases from here only, and compare the published SHA-256 checksums.

The software is provided as is, without warranty. You are responsible for the media and voices you process and for the rights attached to them.

If Dubbit saves you time, a star helps other people find it.

## Credits

Dubbit stands on work by other people. It invokes FFmpeg for media decoding and encoding, and a whisper.cpp build for speech recognition, both as separate executables. Their licences, and those of the Rust crates it uses, are listed in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
