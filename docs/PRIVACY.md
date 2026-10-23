# Privacy

Dubbit is built to work on your computer. It has no accounts, no telemetry and no cloud processing.

## What stays local

- Your video and audio files are read and written on your machine only.
- Transcription, translation, subtitle generation and diagnostics run locally.
- Project folders, checkpoints, subtitles and logs stay in the folders you choose.
- Interface preferences stay on your machine.

## Network access

This build makes no network requests while processing a file. The only outbound connection is a model download that you start yourself, for example with `dubbit models install`. That request goes to the host listed in the model manifest and necessarily reveals your IP address to that host. Nothing about your media is sent.

There are no update checks, analytics or crash reporting in this build. If update checks are added later, they will be optional and described here first.

## Diagnostic reports

`dubbit doctor --report report.json` writes a report on your machine. It contains the operating system, architecture, memory, processor count, graphics adapters, tool versions, installed model identifiers and, when you ask for one after a failure, an error code and title.

It does not contain your media, transcripts, translations, file paths, user name or account information. The report is never sent automatically. You decide whether to attach it to an issue, and you should read it first.

## Model downloads

Models are downloaded from the host in the manifest and verified against a pinned SHA-256 checksum. Downloaded weights have their own licences, documented in `MODEL_LICENSES.md`.

## Contact

Privacy questions can be raised in the repository discussions. Do not include personal data in a public issue.
