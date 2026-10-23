# Development

This page is for people building Dubbit from source. For installing and using the command, read [install.md](install.md) and [user-guide.md](user-guide.md).

## Workspace layout

- `core/` — the Rust crate with settings, media probing, hardware detection, model management, project checkpoints, the subtitle pipeline and the stable API used by any client.
- `apps/cli/` — the `dubbit` binary, a thin client of the core, with argument parsing and console output.
- `models/` — the model manifest. Each entry pins a URL, file size, SHA-256, licence and RAM requirement.
- `docs/` — user documentation and architecture decision records in `docs/adr/`.
- `apps/cli/tests/` — integration tests that run the built binary without a network connection.

The workspace is defined by the root `Cargo.toml`; `Cargo.lock` is committed.

## Required tools

- Rust. `rust-toolchain.toml` pins the version, and `rustup` installs it automatically when you run any Cargo command in the repository.
- FFmpeg and FFprobe, on `PATH` or pointed at by an environment variable.
- A speech recognition executable built from whisper.cpp, which exposes the `whisper-cli` interface and can write JSON output.

Dubbit does not bundle these yet. Without them the pipeline stops with an error code and a next step that names the missing tool.

## Environment variables

| Variable | Use |
| --- | --- |
| `DUBBIT_HOME` | Data folder for models, `config.json` and `state.json`. Defaults to the platform data folder. |
| `DUBBIT_TOOLS_DIR` | Folder searched for FFmpeg, FFprobe and the recognition executable. |
| `DUBBIT_FFMPEG` | Full path to the FFmpeg executable. |
| `DUBBIT_FFPROBE` | Full path to the FFprobe executable. |
| `DUBBIT_ASR` | Full path to the speech recognition executable. |

## Checks

Every change must pass all three from the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Tests run offline. A test that needs a model, FFmpeg or a media sample must be marked ignored and documented next to the test.

## Running the client

```sh
cargo run -p dubbit-cli -- languages
cargo run -p dubbit-cli -- doctor
cargo run -p dubbit-cli -- models list
cargo run -p dubbit-cli -- dub clip.mp4 --to pt
```

The built binary is `target/debug/dubbit`. Use `--json` for machine readable output and `--verbose` for detail on standard error.

## Project folder

A run writes one folder, next to the input unless `--output` says otherwise:

```
clip.dubbit/
  project.json
  audio.wav
  segments-source.json
  segments-translated.json
  subtitles/
    en.srt
    en.vtt
    pt.srt
    pt.vtt
```

`project.json` holds the source fingerprint, the options, the media information, the segment list and one checkpoint per stage. Deleting a stage entry makes the next run redo only that stage. `dubbit project clip.dubbit` prints the current state.

## Conventions

- No comments anywhere: code, tests, configuration, workflows. Names and types carry the meaning.
- Product code, tests, documentation and commits are written in English.
- One changed file per commit, with a short imperative message of at most five words.
- Sign off every commit with the Developer Certificate of Origin using `git commit -s`.
- No dead code, no commented-out blocks, no placeholders.
- Every failure returns a typed error with a stable code, a title, a detail and a next step.

## Adding a model

Edit `models/whisper.json` and add one entry to `models`:

```json
{
  "id": "whisper-base-q5_1",
  "engine": "whisper-cli",
  "file": "ggml-base-q5_1.bin",
  "url": "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-base-q5_1.bin",
  "size": 59707625,
  "sha256": "422f1ae452ade6f30a004d7e5c6a43195e4433bc370bf23fac9cc591f01a8898",
  "license": "MIT",
  "ram_mb": 700,
  "quality": "balanced"
}
```

Rules for an entry:

- `url` points at a specific commit or tag, never at a moving branch.
- `size` is the exact byte length and `sha256` is the digest of that file; downloads are rejected when either differs.
- `license` is the licence of the weights, not of the code that runs them. Non-commercial weights are optional only, clearly marked, and never the default.
- `ram_mb` is the working memory the model needs, used by the hardware profiles.
- `quality` is one of `fast`, `balanced`, `accurate` or `maximum`, and matches the profile names in `core/src/hardware.rs`.

After adding an entry, install it once with `dubbit models install <id>` to confirm the download and the checksum, and add or update the profile mapping if the model should be chosen automatically.

## Licensing

Dubbit is Apache-2.0. Dependencies must be MIT, BSD, Apache-2.0, ISC, or a permissive data licence; GPL and AGPL code is never linked into the binary and may only run as a separate program. Record licence checks in [LICENSES-AUDIT.md](LICENSES-AUDIT.md).
