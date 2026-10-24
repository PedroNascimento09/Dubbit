# Changelog

All notable changes to this project are documented here. The format follows Keep a Changelog, and the project uses semantic versioning.

## Unreleased

### Added

- Desktop window built with Tauri, with a drop area, language and model choices, and a stage list while a job runs.
- Processing queue that persists across restarts, runs one job at a time, and returns an interrupted job to the queue.
- Review editor with editable lines, search, per-line playback, and saving that rewrites every subtitle format.
- Cancellation between stages, with finished stages kept for a retry.
- Automatic model download before the first run, with progress shown in the window.
- Light and dark appearance, remembered between launches, and reduced motion when the system asks for it.
- Recovery screen for an unexpected interface failure, and a deterministic preview mode for interface work.
- `pipeline::save_review` and `pipeline::write_subtitles` as the shared path for writing subtitles from edited lines.
- Windows installer that bundles FFmpeg, FFprobe and the speech recogniser, installs for the current user, and needs no environment variables.
- Bundled tools are found from the application resource folder, so an installed window runs without any setup.
- `models/tools.json` pins the bundled programs by URL, byte size, SHA-256 and licence. `scripts/bundle-tools.py` downloads each archive once, verifies it and stages only the declared programs and libraries, for Windows and Linux.
- The installer ships the licence text of every bundled program in a `licenses` folder, and the FFmpeg build is now the LGPL variant without GPL-only components.
- `scripts/checksums.py` writes SHA-256 side files, and `scripts/sbom.py` writes a CycloneDX bill of materials from the Cargo graph.
- Release workflow builds the Windows installer, a portable Windows package, a Linux `.deb` and AppImage, the command line archives, one checksum file and the bill of materials, and publishes them together.
- Translation quality benchmark: `benchmarks/translation_quality.py` samples a parallel corpus deterministically, translates it with the local model and prints chrF, BLEU and cost per segment, with a line translator that can serve as the command backend.
- ADR 0008 records where translation runs for now, with the measured cost of loading a pair: 83 to 165 MB on disk, 350 to 426 MB of memory and about four seconds to the first line.
- `benchmarks/translation_cost.py` measures that cost for any pair.
- `benchmarks/ollama_shim.py` answers the local model server protocol with the same engine, so the shipped translation stage can be exercised end to end on a machine with no model server installed.
- First published translation numbers for English, Portuguese, Spanish, French and German against English on Tatoeba and Wikimedia, with method, commands and limits in the benchmarks page.
- Cue timing never runs past its own segment, keeps a one second floor when the segment allows it, and lends spare time to cues that are dense for their length. `benchmarks/cue_speed.py` reports characters per second per subtitle file.
- Test that the command line works from a data folder with accents and spaces, and an ignored end-to-end test that subtitles a real media file when the tools are present.

### Changed

- HTTP downloads use the current client release, which drops an unmaintained PEM parsing dependency. Resumed downloads and checksum verification were measured again after the change.
- Revealing a finished project is handled by the desktop backend instead of a plugin, removing a Linux dependency chain that carried unmaintained advisories.
- Path dependencies inside the workspace carry an explicit version, so dependency checks no longer report them as wildcards.

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
- Benchmark tooling: a deterministic corpus builder and a word error rate scorer, with published results for three models.

### Changed

- Recognition output is read from the millisecond offsets the recogniser writes, so cue times no longer drift.
- Long speech is split across several cues instead of being truncated.

### Fixed

- `--resume` after a failed run continues instead of failing on an existing project folder.
- Model downloads fail with a clear code when the certificate store cannot be trusted.

## Notes

- Dubbing, voice synthesis, installers and a graphical interface are not part of this version.
- macOS is untested. Windows and Linux are the supported platforms.
