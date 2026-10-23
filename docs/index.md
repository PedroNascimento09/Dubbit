# Dubbit documentation

Dubbit turns a video or audio file into subtitles, in the original language and
in a translated one, on your own machine.

## Start here

| Page | Read it when |
| --- | --- |
| [Install](install.md) | You want the binary running with the tools it needs |
| [User guide](user-guide.md) | You want to subtitle a file and understand the result |
| [Troubleshooting](troubleshooting.md) | Something failed and you have an error code |
| [Hardware and compatibility](hardware-matrix.md) | You want to know what is tested and what is not |
| [Benchmarks](benchmarks.md) | You want timings and how they were measured |
| [Development](DEVELOPMENT.md) | You want to build, test or contribute |
| [Licence audit](LICENSES-AUDIT.md) | You want to know what Dubbit links and what it runs |

## How the work is organised

A run has four stages and each one writes a checkpoint:

1. Ingest: probe the source and extract 16 kHz mono audio.
2. Transcribe: speech to timed segments in the original language.
3. Translate: segments into the target language, with the glossary protected.
4. Subtitles: sentence-aware cues written as SRT and VTT.

Interrupt the run and start it again with `--resume`: the finished stages are
skipped, and the source file is checked against the checksum stored when the
project was created.

## The project folder

Every run owns one folder, next to the input by default:

```text
<name>.dubbit/
  project.json
  audio.wav
  segments-source.json
  segments-translated.json
  subtitles/<language>.srt
  subtitles/<language>.vtt
```

`project.json` records the source path and checksum, the probe result, the
options and the stage checkpoints. `dubbit project <folder>` prints the same
information in a readable form.

## Two promises this build keeps

- Processing is local. Nothing is uploaded, and no request is made while a file
  is being processed.
- Every failure is a code with a title, a reason and a next step, not a stack
  trace.

## Two promises this build does not make yet

- It does not dub. There is no speech synthesis in this version.
- It does not ship an installer. You build with Cargo.

See the [roadmap](../ROADMAP.md) for what comes next and
[ACCEPTABLE_USE.md](ACCEPTABLE_USE.md) for what the project expects from its
users.
