# Security policy

## Supported versions

Security fixes are provided for the latest published release. Older releases are not maintained.

## Reporting a vulnerability

Do not open a public issue for a security problem. Use the private vulnerability reporting option in the repository Security tab. Include:

- the Dubbit version;
- the operating system and hardware;
- the steps that reproduce the problem;
- the impact you believe it has;
- the diagnostic report from `dubbit doctor --report report.json` after reviewing it for personal paths and file names.

Reports are acknowledged as soon as possible and handled privately until a fix is available. Credit is given in the release notes unless you ask to stay anonymous.

## Scope

In scope:

- the Dubbit source code and release binaries;
- project files and checkpoints written by Dubbit;
- model manifest verification and download handling;
- the local command execution paths used by auxiliary engines.

Out of scope:

- vulnerabilities in third-party media tools, operating systems or model weights;
- problems that require an already compromised machine;
- denial of service caused by intentionally malformed media on a machine with no resource limits.

## What Dubbit does to reduce risk

- Model files are pinned by URL, size and SHA-256 in the manifest, and verified before use.
- Processing is local. Dubbit makes no network requests while working on a file.
- No telemetry, accounts or cloud storage are used.
- Release artefacts are published with checksums, and dependencies are audited in continuous integration.

Please do not include private media, personal information or credentials in a report.
