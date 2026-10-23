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

State: partly measured. A 7.36 s clip transcribed in about 1.0 s wall time with
`whisper-tiny-q5_1` on a 12-core Windows machine, using the CPU path. Memory was
not artificially capped, the file was short, and accuracy was not measured.

What closes it: a ten minute file, a hard 8 GB memory limit, peak memory
recorded, and the result published in [benchmarks](../benchmarks.md).

## S4 Translation quality

Question: is a lightweight offline translator good enough for the launch
languages, and what does it cost to run?

State: open. Translation backends exist (passthrough, a local model server, and
an external command), but no quality metric has been produced. No language has a
grade.

What closes it: a fixed evaluation set per language pair, an automatic metric
computed by a script in the repository, and a written review by a speaker of
each language.

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
