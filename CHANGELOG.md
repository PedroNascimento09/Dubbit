# Changelog

All notable changes to this project are documented here. The format follows Keep a Changelog, and the project uses semantic versioning.

## 0.1.0 - 2026-10-23

### Added

- Command line client `dubbit` with `dub`, `models`, `doctor`, `languages`, `config` and `project` commands.
- Subtitle pipeline that extracts audio, transcribes speech and writes SRT and VTT files, with source and translated tracks.
- Sentence-aware subtitle layout: at most 42 characters per line, at most two lines per cue, and cue times proportional to text length.
- Project folders with `project.json`, per-stage checkpoints and stage-level resume, including a check that the source media has not changed.
- Model manager with a pinned manifest, resumable downloads, SHA-256 verification, install, remove, verify and path commands.
- Four recognition model sizes from a single quantised family, selected by hardware profile or by hand.
- Hardware detection for processor count, memory, platform and graphics adapters, and three profiles: light, balanced and max.
- Translation backends: passthrough, a local HTTP model server, and an external command reading on standard input and writing on standard output.
- Glossary support with case-insensitive matching, longest term first, and protection across translation.
- Diagnostic report written by `dubbit doctor --report`, without media, transcripts or personal paths.
- Typed error catalogue with stable codes, a human title, a detail and a next step, plus JSON error output.

### Changed

- Recognition output is read from the millisecond offsets the recogniser writes, so cue times no longer drift.
- Long speech is split across several cues instead of being truncated.

### Fixed

- `--resume` after a failed run continues instead of failing on an existing project folder.
- Model downloads fail with a clear code when the certificate store cannot be trusted.

## Notes

- Dubbing, voice synthesis, installers and a graphical interface are not part of this version.
- macOS is untested. Windows and Linux are the supported platforms.
