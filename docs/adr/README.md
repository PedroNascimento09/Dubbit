# Architecture decision records

A decision record states one choice, the context that forced it, and the cost accepted with it. Records are immutable once accepted; a later record supersedes an earlier one instead of rewriting it.

| Record | Decision |
| --- | --- |
| [0001](0001-rust-workspace.md) | Rust workspace: one core library crate and a thin command line crate |
| [0002](0002-apache-license.md) | Apache-2.0 for all project code |
| [0003](0003-auxiliary-binaries.md) | Media and recognition tools run as separate executables |
| [0004](0004-project-format.md) | One folder per project, described by `project.json` |
| [0005](0005-model-manifests.md) | Models are pinned in a manifest and downloaded by the user |
| [0006](0006-language-first.md) | English first for code, commits and technical documentation |
