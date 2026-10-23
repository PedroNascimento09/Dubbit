# Contributing

Thank you for helping Dubbit. The project is written in Rust with a small, typed core and a thin command line client.

## Before you start

- Open an issue for a large change before writing it, so the design can be agreed first.
- Keep each pull request focused on one problem.
- Read the code around the part you change and follow its existing patterns.
- Product code, tests, documentation and commit messages are written in English.
- Code and configuration contain no comments. Clear names and types carry the meaning.

## Development setup

Install the pinned Rust toolchain with `rustup toolchain install` from the repository root. The toolchain file selects the right version automatically.

Install FFmpeg and FFprobe and make them available on `PATH`, or set `DUBBIT_TOOLS_DIR` to the folder that contains the executables. Speech recognition needs a whisper.cpp `whisper-cli` build on `PATH`, or its full path in `DUBBIT_ASR`.

## Checks

Every change must pass:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Tests must run without a network connection. A test that needs a model or a large media sample must be marked ignored and documented.

## Commits and sign-off

Dubbit uses the Developer Certificate of Origin. Sign every commit:

```sh
git commit -s
```

The sign-off certifies that you wrote the change or have the right to submit it under the project licence. No contributor licence agreement is required.

Write commit messages in the imperative, keep them short, and explain why the change exists rather than restating the diff.

## Pull requests

A pull request should describe the problem, the chosen solution and the checks you ran. Add or update tests for behaviour that can break. Keep documentation in step with user-visible changes. Mention anything you did not verify.

## Licence

By contributing you agree that your contribution is licensed under Apache-2.0, the licence of this project. The name, logo and mascot are not covered by the code licence.
