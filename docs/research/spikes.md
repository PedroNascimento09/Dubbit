# Technical spikes

Spikes answer a question that could change the architecture before more code is
written. Each entry records the question, the state, and what would count as an
answer. No result is claimed that was not produced by a run.

## S1 Packaging

Question: can a single Rust binary plus auxiliary executables be packaged for
Windows and Linux without manual steps?

State: open. Installers are not built yet. The binary is a normal Cargo release
build and has no runtime dependency other than the media and recognition
executables.

What closes it: a clean virtual machine installs the artefact with no toolchain
and subtitles a sample file offline, on Windows and on Linux.

## S2 CPU performance under 8 GB of RAM

Question: does the light profile finish a longer file on a machine limited to
8 GB, with no graphics acceleration?

State: closed for the transcription path. Three runs on 2026-10-23, release
build, Windows 11, 12 logical cores, 15 GB of RAM, a 13 min 37 s (817 s)
synthesised speech file, inside a Windows job object that caps the process and
the job at 8 GiB, including the recognition child process:

| Model | Profile | Wall time | Ratio | Peak job memory |
| --- | --- | --- | --- | --- |
| `whisper-tiny-q5_1` | light | 47.64 s | 17.2x | 731.5 MB |
| `whisper-base-q5_1` | light, forced | 69.79 s | 11.7x | 789.2 MB |
| `whisper-small-q5_1` | balanced | 200.44 s | 4.1x | 1055.0 MB |

All runs exited 0 and produced 223 to 224 cues covering the whole file. Peak
memory stayed far below the cap, so the cap proves headroom, not the behaviour
of an out-of-memory failure. Results and commands are in
[benchmarks](../benchmarks.md).

Residual limits: synthesised single-voice speech, no translation stage, one
machine, no Linux or macOS run. Accuracy is measured separately in
[benchmarks](../benchmarks.md), Measurement 3.

## S4 Translation quality

Question: is a lightweight offline translator good enough for the launch
languages, and what does it cost to run?

State: partly measured. Translation backends exist (passthrough, a local model
server, and an external command). The first automatic numbers exist for English
and Portuguese with the OPUS-MT weights as packaged by Argos Translate 1.9 on
CTranslate2: chrF 68.45 and BLEU 49.18 for en → pt on 200 Tatoeba sentences,
76.87 and 65.02 for pt → en, and 71.06 and 52.49 for en → pt on 200 Wikimedia
sentences, 71.67 and 52.36 the other way. Cost is 0.06 to 0.19 seconds per
sentence on CPU, which is negligible beside transcription. Methods, commands
and limits are in [benchmarks](../benchmarks.md), Measurement 4.

The Tatoeba numbers are optimistic because those sentences are very likely in
the model's training data. The Wikimedia numbers are the fairer signal. No
language has a grade yet, no human has read the output, and no other pair has a
number.

What closes it: a written review by a speaker per launch pair, numbers for the
remaining pairs, and a decision on whether the product ships these weights
in-process or keeps the existing server and command backends.

Residual risk: the measurement ran the model directly, not through the product.
The shipped path for a local translator command takes no arguments, so a Python
script needs an executable wrapper on Windows; scripting the product against
these weights is still to be done.

## S7 Voice synthesis in Rust

Question: is there a voice engine with a permissive licence that reaches
acceptable quality on CPU?

State: not started. Voice synthesis is out of scope until the dubbing milestone.

What closes it: at least one voice per launch language, latency and memory
measured on the light profile, and the weights documented in the model licences.

## How a spike result is recorded

Each closed spike adds a section with the date, the exact command, the raw
output, the machine and the decision. A decision that changes the architecture
also gets an entry in `docs/adr/`.
