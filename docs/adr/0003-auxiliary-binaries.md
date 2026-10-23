# 0003: Auxiliary tools run as separate executables

- Status: Accepted
- Date: 2026-10-23

## Context

Two jobs sit outside the Rust code: reading and writing media, and speech recognition from a quantised model. Both have mature command line implementations that are well understood and easy to replace.

Linking those projects as libraries would pull their build systems into the Rust build, complicate the licence story, and make a compiler upgrade a media-tooling upgrade as well. Distributing them as separate executables keeps each piece replaceable.

## Decision

The pipeline invokes auxiliary tools as processes and exchanges plain files with them.

- `ffprobe` describes the input: duration, streams, codecs, sample rate, channel count.
- `ffmpeg` extracts mono 16 kHz PCM audio into the project folder, and writes nothing else during the subtitle pipeline.
- `whisper-cli` reads that audio and writes a JSON transcript with millisecond offsets, which the core parses.
- Translation backends follow the same shape. A command backend reads text on standard input and writes the translation on standard output, with the language pair in environment variables.

Resolution order is explicit and testable: a `DUBBIT_*` environment variable, then the data folder's `bin` directory, then the folder of the running executable, then the working directory's `.tools/bin`, then `PATH`. A missing tool produces one error code with the name of the tool and a concrete next step, never a panic or a silent fallback.

Nothing is linked against a copyleft library. The boundary is the process and the file, so the core crate stays under its own permissive licence.

## Consequences

- Users can supply their own builds, including hardware-specific ones, without rebuilding Dubbit.
- The licence boundary is simple to explain and to audit.
- Tools can be upgraded independently of the Rust code.
- The cost is paid per run: process startup, a temporary file, and no streaming between stages. On short files this is measurable but small; on long files the audio stage dominates.
- Failures are less precise. A tool that dies without a readable message leaves only its last output line, so the error type always carries that line and names the tool.
- Windows needs care with executable extensions and paths containing spaces, both handled in one resolution helper rather than at each call site.
- Distribution must eventually bundle the tools per platform. Until then, `dubbit doctor` reports what is missing and the install guide explains the one-time setup.

## Alternatives considered

- Linking a media library into the crate: fewer moving parts, more build fragility and a harder licence story.
- Linking the recognition library directly: faster for repeated runs, but it pins the model format to this binary and removes the option of using a faster local build.
- A long-running helper service: avoids startup cost, adds a lifecycle, a socket and an orphan-process problem for a tool that usually runs once.
