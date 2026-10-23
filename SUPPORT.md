# Support

Dubbit is maintained by one person. Answers come as fast as time allows, usually within a few days, and there is no guaranteed response time or commercial support.

## Where to ask

- Questions, ideas and results: use Discussions. This is the right place for "how do I", "is this expected" and sharing what you produced.
- Reproducible defects: open an issue with the bug template. One problem per issue.
- Security problems: use the private advisory form linked from the issue chooser. Never open a public issue for a vulnerability.

## What to include

A report without details usually needs a round trip before anyone can help. Include:

- the Dubbit version, from `dubbit --version`;
- the operating system and version;
- the hardware summary from `dubbit doctor`;
- the exact command you ran, with your own file paths replaced by `example.mp4`;
- the error code and title, for example `DUB-2101 No speech detected`, not only the message;
- what you expected and what happened.

`dubbit doctor --report report.json` writes a diagnostic report. Read it before attaching it: it lists tool paths, installed model identifiers and hardware, but no media, transcripts or translations. It is never sent automatically. Add the failure details yourself.

For a crash or a wrong result, a short media clip that reproduces the problem helps more than any description. Use material you have the right to share, and cut it to a few seconds.

## What is not supported

- Media you do not own or do not have permission to process.
- Installing, configuring or debugging FFmpeg and speech recognition binaries on your machine.
- Hardware acceleration that is not listed as verified in the compatibility documentation.
- Requests to make the software bypass copyright, platform terms or protection measures.
- Private one-to-one help over email or chat.

## Before you ask

Run the three checks in [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) if you build from source, and read [docs/troubleshooting.md](docs/troubleshooting.md). Many failures come from a missing tool, a model that was not downloaded, or a project folder that already exists; each has a message with a next step and an error code.
