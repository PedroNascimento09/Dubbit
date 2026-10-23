# User guide

Dubbit turns a video or audio file into subtitles, and can translate them into another language. Everything runs on your machine. This build produces subtitles only; it does not produce dubbed audio.

## One command

```sh
dubbit dub video.mp4 --from en --to pt
```

That command does four things in order: extracts the audio, transcribes the speech, translates each line, and writes subtitle files. Each finished stage is saved, so a failure later does not throw the work away.

The project folder is created next to the input and named after it, `video.dubbit` here. Choose another folder with `--output`:

```sh
dubbit dub video.mp4 --to pt --output /data/subtitles/video
```

Transcribe without translating:

```sh
dubbit dub video.mp4 --no-translate
```

`--from` is optional. When it is omitted, the recogniser detects the language of the speech.

## What you get

Inside the project folder:

- `subtitles/en.srt`, `subtitles/en.vtt` for the spoken language
- `subtitles/pt.srt`, `subtitles/pt.vtt` for the translated language, when translating
- `project.json` with the source checksum, the settings and the stages that finished
- `audio.wav` with the extracted mono audio
- `segments-source.json` and `segments-translated.json` with the text and timings

Cues follow sentences, use at most 42 characters per line and at most two lines per cue. The time of each cue is proportional to the length of its text inside the original segment.

## Programs and progress

```sh
dubbit dub talk.mkv --to es
```

```text
(1/4) Extracting audio
(2/4) Transcribing speech
(3/4) Translating subtitles
(4/4) Writing subtitles
12 segments in 41.3s
  talk.dubbit/subtitles/en.srt
  talk.dubbit/subtitles/en.vtt
  talk.dubbit/subtitles/es.srt
  talk.dubbit/subtitles/es.vtt
```

Add `--json` to print one machine readable object instead:

```sh
dubbit dub talk.mkv --to es --json
```

Add `--verbose` for detail about each stage.

## Models

The recognition model decides speed, memory use and accuracy. Dubbit ships no weights; you download the ones you choose.

```sh
dubbit models list
dubbit models install whisper-tiny-q5_1
dubbit models install whisper-large-v3-turbo-q5_0
dubbit models verify
dubbit models remove whisper-large-v3-turbo-q5_0
```

| Model | Size on disk | Suggested memory |
| --- | --- | --- |
| `whisper-tiny-q5_1` | about 31 MB | 400 MB |
| `whisper-base-q5_1` | about 57 MB | 700 MB |
| `whisper-small-q5_1` | about 181 MB | 1.5 GB |
| `whisper-large-v3-turbo-q5_0` | about 547 MB | 4 GB |

All four are MIT licensed. Downloads resume and are checked against a pinned SHA-256 before use.

## Hardware profiles

A profile picks a default model and a memory budget. `auto` inspects the machine and chooses for you.

| Profile | Model | Memory budget |
| --- | --- | --- |
| `light` | `whisper-tiny-q5_1` | 4 GB |
| `balanced` | `whisper-small-q5_1` | 8 GB |
| `max` | `whisper-large-v3-turbo-q5_0` | 16 GB |

```sh
dubbit dub video.mp4 --profile light
```

An explicit `--model` wins over the profile. `--threads` sets how many processor threads the recogniser uses; zero means the profile choice.

## Translation backends

`--translator` chooses how text is translated.

- `auto` (default) checks for a local model server on `127.0.0.1:11434`. If one answers, it is used; otherwise text passes through unchanged, which is useful for transcribing in the original language.
- `passthrough` never translates. The target language still gets subtitle files, with the original text.
- `ollama` requires the local model server at the configured host and model. Configure it with `dubbit config set ollama-host` and `dubbit config set ollama-model`.
- `command` runs your own program. It reads the text on standard input and writes the translation on standard output. Dubbit sets `DUBBIT_SOURCE` and `DUBBIT_TARGET` in its environment.

Example of a command backend on Linux:

```sh
dubbit config set translator-command /home/me/bin/translate-line
dubbit config set translator command
```

The server or program is only used for text. Your media never leaves the machine unless the program you configured sends it somewhere, which is your choice.

## Glossaries

A glossary pins names and terms to one wording. It is a CSV file with two columns:

```csv
source,target
Dubbit,Dubbit
release candidate,candidato de lançamento
```

Rules:

- one term per row, `source` then `target`
- a header row is optional and detected
- lines starting with `#` are ignored
- matching is case-insensitive
- the longest matching term wins

Use it for one run:

```sh
dubbit dub lesson.mp4 --to pt --glossary terms.csv
```

Or keep it in the settings:

```sh
dubbit config set glossary /home/me/terms.csv
```

Terms are protected before translation and restored afterwards, so a model does not rename a product or a person.

## Resuming

If the command stops, run the same command again with `--resume`:

```sh
dubbit dub lesson.mp4 --to pt --resume
```

Finished stages are skipped. Dubbit verifies the checksum of the source file first, and refuses to continue if the media changed.

Without `--resume`, an existing project folder is never overwritten. Dubbit tells you it exists and stops.

## Inspecting a project

```sh
dubbit project lesson.dubbit
```

The output lists the source, the languages, the model, the formats and which stages finished, plus the subtitle files on disk. `--json` prints the same information as one object.

## Settings

```sh
dubbit config show
dubbit config set profile balanced
dubbit config set asr-model whisper-small-q5_1
dubbit config set threads 6
dubbit config path
```

Keys: `profile`, `asr-model`, `translator`, `ollama-host`, `ollama-model`, `translator-command`, `glossary`, `threads`. Invalid values are rejected with a code and a next step.

## Languages

```sh
dubbit languages
```

Codes are lower case, with an optional region such as `pt-BR`. A region is normalised, and matching for the source and target ignores it.

## Using the output

SRT and VTT are accepted by common players and editors. Import the file into a video editor, or play it next to the video with the same name.

If the timing looks wrong, the usual cause is a wrong source language. Re-run with `--from` set explicitly.

## Privacy and limits

No accounts, no telemetry, no upload. The only network request is a model download you start. The recogniser runs on the CPU; a GPU is not used by this build. Dubbed audio, voice features and a graphical application are not part of this build.
