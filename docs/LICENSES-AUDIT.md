# Licence audit

This audit decides what Dubbit may link, what it may run as a separate program,
and what it may not ship. It was last verified on 2026-10-24 by reading the
licence metadata of every crate in `Cargo.lock` and the licence files of the
separate programs. It is an engineering record, not legal advice.

## Rules

- Linked code must be permissive and compatible with Apache-2.0: MIT,
  Apache-2.0, BSD, ISC, Unicode, CDLA-Permissive and Unlicense all qualify.
- Copyleft code (GPL or AGPL) is never linked. It may only run as a separate
  executable invoked over the command line.
- Non-commercial weights are never a default. They may only be added as an
  explicit optional download with a visible warning, and only with the owner's
  approval.
- Every model entry carries a URL pinned to a commit, a byte size, a SHA-256 and
  a licence.

## Components in use

| Component | Version | Licence | Commercial use | Distributed by Dubbit | Decision |
| --- | --- | --- | --- | --- | --- |
| Rust standard library and crates in `Cargo.lock` | current | MIT, Apache-2.0, ISC, BSD-3-Clause, Unicode-3.0, CDLA-Permissive-2.0, Unlicense | Yes | Linked into the binary | Accepted |
| FFmpeg and FFprobe | N-127957-g935f29eaae, LGPL build | LGPL-3.0-or-later | Yes, with conditions | Yes, separate executable in the installer and portable package | Accepted. A GPL build must never be bundled; `models/tools.json` pins the LGPL archive and the licence text ships in `resources/licenses/` |
| whisper.cpp `whisper-cli` | 1.9.5, build b5454 | MIT | Yes | Yes, separate executable in the installer and portable package | Accepted, with its MIT text bundled |
| Whisper quantised weights | as listed in the manifest | MIT | Yes | No, downloaded on request | Accepted |
| Rustls, ring, webpki-roots | current | Apache-2.0, ISC, MIT, CDLA-Permissive-2.0 | Yes | Linked | Accepted |

## Candidates not yet integrated

Nothing in this table is in the build. Each row records the state of the
assessment so the decision is not repeated from memory later.

| Component | Licence | Commercial use | State |
| --- | --- | --- | --- |
| ONNX Runtime | MIT | Yes | Candidate for acceleration; needs a packaging proof before adoption |
| Silero VAD | MIT | Yes | Candidate for speech detection |
| Kokoro weights | Apache-2.0 | Yes | Candidate for voice synthesis, out of scope until dubbing |
| Piper voices | voice by voice, often permissive | Depends on the voice | Every voice must be read and recorded before use |
| OPUS-MT weights | MIT or CC-BY depending on the pair | Yes, with attribution where CC-BY | Candidate for offline translation |
| CTranslate2 | MIT | Yes | Reference only; the runtime path is Rust |
| NLLB-200 weights | CC-BY-NC-4.0 | No | Optional only, with a warning, and not a default |
| XTTS weights | Non-commercial | No | Excluded from the default build |
| F5-TTS and similar weights | Often CC-BY-NC | No | Excluded from the default build |
| pyannote weights | Gated terms | Restricted | Avoided as a default |
| espeak-ng | GPL-3.0 | Yes, with copyleft conditions | Separate executable only, and only if a permissive alternative fails |
| Demucs weights | MIT code, weights for scientific use | No, not for redistribution as part of a commercial product | Ship only as an optional, user-requested download with the licence shown; never bundled, never the default |
| GTCRN | MIT, weights included | Yes | Accepted candidate for speech enhancement |

## Weak copyleft inside the desktop window

The desktop window pulls a small set of Mozilla Public License 2.0 crates:
`cssparser`, `cssparser-macros`, `selectors`, `dtoa-short` and `option-ext`.
They arrive below `tauri-utils` and `dirs`, and none of them is modified.
MPL-2.0 is file-level copyleft: linking it with Apache-2.0 code is allowed,
and the obligation is to publish changes to those files, which do not exist.
The licence is allowed in `deny.toml` for that reason and for no other. GPL and
AGPL code remains excluded, and no GPL or AGPL binary is linked.

## Checks

- `cargo deny check licenses` runs in continuous integration against
  `deny.toml`, which allows only the permissive licence set above.
- `cargo deny check` also covers advisories, bans and sources. One advisory is
  ignored: `RUSTSEC-2024-0370` for `proc-macro-error`, which reaches the build
  only through the GTK bindings that Tauri uses on Linux, runs as a build-time
  macro and has no newer upstream release. The ignore is recorded with its
  reason in `deny.toml`.
- The model manifest is reviewed by hand when a model is added; a checksum or
  licence mismatch fails the review.
- The tool manifest works the same way: every entry pins a URL, a byte size, a
  SHA-256 and a licence, and `scripts/bundle-tools.py` refuses to stage anything
  that does not match. The LGPL source for the bundled FFmpeg build stays
  available at the pinned tag named in its licence folder.
- Release notes record any licence change and the date it was verified.
