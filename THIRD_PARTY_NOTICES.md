# Third-party notices

Dubbit is Apache-2.0 licensed. It depends on the work below, which keeps its own
licence. This page records what is used, under which terms, and whether it is
linked into Dubbit or run as a separate executable.

Verified 2026-10-24 against the licence metadata of each published package and
the licence files shipped with each program.

## Separate executables

Dubbit invokes these as child processes and exchanges files or standard input
and output with them. They are not linked into the Dubbit code.

| Program | Used for | Licence | Distribution note |
| --- | --- | --- | --- |
| FFmpeg and FFprobe | Probing media and extracting audio | LGPL-3.0-or-later | The installer and the portable package ship build N-127957-g935f29eaae, an LGPL build without GPL-only components, with its licence text and source location in `apps/desktop/src-tauri/resources/licenses/`. The repository does not contain the binaries: `scripts/bundle-tools.py` fetches them from the pinned entry in `models/tools.json` and checks size and SHA-256. |
| whisper-cli | Speech recognition | MIT | The installer and the portable package ship build b5454, version 1.9.5, with its MIT text in the same folder. Fetched the same way from the pinned manifest. |

The `dub` pipeline communicates with both through the command line only. No
FFmpeg or whisper.cpp headers, objects or static libraries are linked into the
Dubbit binaries.

## Rust crates

All crates below are permissively licensed and compatible with Apache-2.0.

| Crate | Version | Licence |
| --- | --- | --- |
| serde, serde_json | 1.0.x | MIT OR Apache-2.0 |
| sha2, digest, generic-array, block-buffer, crypto-common, cpufeatures | 0.10.x | MIT OR Apache-2.0 |
| ureq | 2.12.x | MIT OR Apache-2.0 |
| csv, csv-core | 1.4.x / 0.1.x | Unlicense OR MIT |
| tempfile, fastrand, getrandom | 3.x / 2.x / 0.2.x | MIT OR Apache-2.0 |
| clap, clap_builder, clap_derive, clap_lex | 4.6.x | MIT OR Apache-2.0 |
| rustls, rustls-native-certs, rustls-pemfile, rustls-pki-types | 0.23.x / 0.7.x / 2.2.x / 1.15.x | Apache-2.0 OR ISC OR MIT |
| ring | 0.17.x | Apache-2.0 AND ISC |
| untrusted, rustls-webpki | 0.9.x / 0.103.x | ISC |
| webpki-roots | 0.26.x, 1.0.x | CDLA-Permissive-2.0 |
| schannel | 0.1.x | MIT |
| url, idna, percent-encoding, form_urlencoded | current | MIT OR Apache-2.0 |
| icu_* and zerovec, yoke, tinystr, writeable, litemap, potential_utf, zerotrie, zerofrom, stable_deref_trait | current | Unicode-3.0 |
| windows-sys, windows-targets, windows-link, windows_x86_64_gnu | 0.52.x / 0.61.x | MIT OR Apache-2.0 |
| syn, quote, proc-macro2, unicode-ident | current | MIT OR Apache-2.0 |
| itoa, ryu, memchr, once_cell, bitflags, smallvec, log, shlex, strsim, subtle, zeroize, base64, cc, cfg-if, libc, version_check | current | MIT, Apache-2.0, BSD-3-Clause or Unlicense as declared by each package |

The precise set and versions for a build are recorded in `Cargo.lock`, which is
committed. A release also publishes a software bill of materials.

No GPL or AGPL library is linked into Dubbit. Copyleft components may only run as
separate executables, which keeps the licence boundary intact.

## Model weights

The speech models are downloaded on request, not shipped. Their sources,
checksums and licences are in [MODEL_LICENSES.md](MODEL_LICENSES.md).

## Adding a dependency

A new crate must be permissive (MIT, Apache-2.0, BSD, ISC, Unicode or similar)
and compatible with Apache-2.0. A copyleft crate needs an architecture decision
record and a process boundary before it is merged. Run the licence check in
continuous integration before opening the pull request.
