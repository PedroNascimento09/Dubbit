# Licence audit

This audit decides what Dubbit may link, what it may run as a separate program,
and what it may not ship. It was last verified on 2026-10-23 by reading the
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
| FFmpeg and FFprobe | external build | LGPL-2.1-or-later, or GPL-2.0-or-later for some builds | Yes, with conditions | No, supplied by the user today | Accepted as a separate program; release builds must use an LGPL build |
| whisper.cpp `whisper-cli` | external build | MIT | Yes | No, supplied by the user today | Accepted as a separate program |
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
| Demucs | MIT | Yes | Candidate for background separation |

## Checks

- `cargo deny check licenses` runs in continuous integration against
  `deny.toml`, which allows only the permissive licence set above.
- The model manifest is reviewed by hand when a model is added; a checksum or
  licence mismatch fails the review.
- Release notes record any licence change and the date it was verified.
