# Licence texts for bundled programs

These files ship inside the installer and the portable package, next to the
programs they cover. They are separate executables that Dubbit starts as child
processes; nothing here is linked into the Dubbit binaries.

| File | Covers | Version | Licence |
| --- | --- | --- | --- |
| `FFmpeg-LICENSE.txt` | `ffmpeg` and `ffprobe` | N-127957-g935f29eaae, LGPL build without GPL-only components | LGPL-3.0-or-later |
| `whisper-cli-LICENSE.txt` | `whisper-cli` and the recognition libraries beside it | 1.9.5, build b5454 | MIT |

## Where the exact source is

- FFmpeg: commit `935f29eaae` of the upstream FFmpeg repository, built with the
  configuration printed by `ffmpeg -version`. The build is the `win64-lgpl` and
  `linux64-lgpl` archive of the `autobuild-2026-10-23-13-04` tag. The LGPL
  requires that the corresponding source stays available; that tag, its build
  logs and its source archives are the offer.
- whisper-cli: tag `b5454` of the upstream whisper.cpp repository, matching
  release 1.9.5.

## How they arrive

`scripts/bundle-tools.py` reads `models/tools.json`, downloads each pinned
archive, checks its size and SHA-256, extracts it and copies only the declared
programs and libraries into `resources/bin`. The licence texts in this folder
are the ones shipped inside those archives. A new version means a new manifest
entry, a new checksum and a fresh copy of these texts.
