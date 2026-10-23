# 0005: Models are pinned in a manifest and downloaded by the user

- Status: Accepted
- Date: 2026-10-23

## Context

The recognition models are tens to hundreds of megabytes. They cannot live in a Git repository, they are hosted elsewhere, and the host may publish a new file at the same URL. A model that changes silently would make a finished project impossible to reproduce and could break the parser with no visible cause.

The download is also the only network request the product makes, so it must be the user's decision and must be visible in the interface.

## Decision

Models are described in `models/whisper.json` and fetched at the user's request.

- Every entry has an identifier, engine name, file name, download URL, exact byte size, SHA-256, licence, RAM estimate and a quality label.
- The URL points at an immutable revision of the upstream repository, not at a moving branch.
- The download resumes with a ranged request, writes to a partial file, verifies the length and the digest, and only then renames the file into place.
- The installed state is recorded in the data folder, together with the digest that was verified. A model whose recorded digest no longer matches the manifest is treated as not installed.
- The pipeline never downloads on its own. A missing model produces an error with the install command to run, and the model manager is the only place that starts a transfer.
- `dubbit models verify` rehashes files on disk; `dubbit models list` shows size, memory estimate, licence and state.

Models are never committed to the repository, and the licence of each model is documented separately from the licence of the code.

## Consequences

- A project can be reproduced exactly while the pinned revision stays available.
- Interrupted downloads resume instead of restarting, which matters on slow or metered connections.
- Corruption is caught before a model is used, with a distinct error code and a next step.
- The cost is a network dependency at install time. A machine with no connection cannot obtain a model unless it copies a folder in by hand, which the documentation describes.
- Checksum strictness has a sharp edge: if upstream republishes a different file at the pinned revision, the download fails rather than silently accepting it. Fixing that means a new manifest entry, reviewed like code.
- Every model addition is a manifest change with a licence check, which keeps the audit honest but adds a review step for each new language or quality tier.

## Alternatives considered

- Ship the model in the installer: truly offline, but a much larger artefact and a redistribution obligation per model.
- Download the latest file at a moving URL: simpler manifest, no reproducibility and no protection against a republished file.
- Fetch from a project-owned mirror: full control, and an ongoing hosting bill the project does not want.
