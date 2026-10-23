# 0004: One folder per project, described by project.json

- Status: Accepted
- Date: 2026-10-23

## Context

A run of the pipeline costs minutes to hours, and users close laptops, run out of battery and cancel jobs. A failed run that throws away an hour of recognition is the fastest way to lose a user. The work also has to be inspectable: a user should be able to see what was recognised before trusting a translation.

State therefore has to live on disk, in a form a human can read and a future version can migrate.

## Decision

Each run writes one folder containing everything needed to reproduce its outputs.

- `project.json` carries a schema number, the source path, its byte size and SHA-256, the media probe result, the pipeline options (languages, model, translator, formats), the segments, and a record per finished stage.
- `audio.wav` is the extracted mono track used for recognition.
- `segments-source.json` and `segments-translated.json` are the segment checkpoints written after each of those stages.
- `subtitles/<language>.<ext>` holds the delivered files, one per language and format.
- A diagnostic report may be written anywhere the user asks for; the project folder does not create one on its own.

Every stage checks its own record before running. A stage that is marked done and whose output file still exists is skipped. Ingest is skipped only when the audio file is present as well, so a deleted artefact is regenerated rather than trusted.

Resuming verifies the source fingerprint first. If the media changed since the project was created, the run stops with a clear error instead of mixing transcripts from two different files. A project written by a newer schema is refused rather than partially read.

Nothing in the folder is a cache that may be silently discarded. Deleting a stage artefact is the documented way to force that stage to run again.

## Consequences

- An interrupted run continues from the stage that failed, at the cost of redoing only that stage.
- Outputs are reproducible from `project.json` plus the source file and the pinned model. The model is referenced by identifier, never copied into the folder.
- The folder is inspectable. A user can read the recognised text, compare it with the translation, and hand both to another tool.
- Folders grow: audio is stored uncompressed, and two segment checkpoints duplicate the transcript. For a two hour video this is hundreds of megabytes.
- Schema changes need migration code and a test that opens a project written by the previous version. Until the first public release, the number can change; afterwards it can only grow.
- Two processes working on the same folder are not supported. There is no lock file yet, and concurrent runs would interleave writes.

## Alternatives considered

- A single archive file: tidier to share, hostile to inspection and to partial regeneration.
- A database: better queries, worse portability, and the project stops being readable without the tool.
- Recomputing everything on every run: simplest code, unacceptable wait on a second attempt.
