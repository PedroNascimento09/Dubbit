# 0001: Rust workspace with a core library and a thin client

- Status: Accepted
- Date: 2026-10-23

## Context

Dubbit has to install as one artefact per platform, work without a language runtime or package manager, and stay usable on modest hardware. The same pipeline must later serve a desktop interface, the command line and any local automation, without three copies of the logic drifting apart.

The pipeline is also mostly orchestration: probe the media, extract audio, call a recogniser, call a translator, write subtitles, record progress. That work benefits from a strict compiler and cheap static binaries far more than from a dynamic runtime.

## Decision

The repository is one Cargo workspace with two members.

- `core/`, published as the `dubbit-core` library crate, owns the pipeline, the engines, the project checkpoint, the model manager, settings, hardware detection, diagnostics and the error type.
- `apps/cli/`, the `dubbit-cli` crate, produces the `dubbit` binary. It parses arguments, reads settings, prints progress and maps `DubbitError` to an exit code. It contains no pipeline logic.
- A desktop shell is added later under `apps/` and calls the same core library. It does not fork the pipeline.

The core is written so that a client can render progress as it happens: pipeline stages report through a callback, and every stage writes its result to disk before the next one starts.

Python is not part of the distributed product. A local virtual environment may exist for one-off conversion or research work, but it is ignored by version control and never shipped.

## Consequences

- A single statically linked binary per platform, with no interpreter to install.
- The engine surface is typed, so a broken call is a compile error rather than a runtime surprise in a user's project.
- The desktop shell inherits resumability, diagnostics and error codes for free.
- The cost is real: media and machine learning tooling in Rust is thinner than in Python, so some work is written where a binding may not exist. More code is written by hand, and some features wait for a crate rather than a package.
- Contributors need a Rust toolchain. The pinned toolchain file and a short setup section in the contributing guide are the whole requirement.

## Alternatives considered

- A Python application with a packaged runtime: faster to prototype, but a large installer, fragile dependency resolution and a slow first start.
- A single crate holding both the library and the binary: fewer files, but the boundary between engine and client disappears and the future desktop shell has nothing to depend on.
- A separate service process for the pipeline: cleaner isolation, but an extra protocol, an extra failure mode and a slower start for a tool that mostly sits idle between runs.
