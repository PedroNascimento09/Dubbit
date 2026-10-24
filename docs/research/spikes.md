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
server, and an external command). Automatic numbers now cover the five launch
candidates, each way against English, with the OPUS-MT weights as packaged by
Argos Translate 1.9 on CTranslate2 at float32. On 200 Wikimedia sentences the
chrF is 71.06 for en → pt, 71.67 for pt → en, 70.46 for en → es, 71.71 for
es → en, 68.99 for en → fr, 68.43 for fr → en, 62.51 for en → de and 66.30 for
de → en; BLEU follows the same order. Tatoeba scores are three to seven points
higher for every pair. German is the weakest pair and the only one under 65
chrF on real text. Cost is 0.05 to 0.33 seconds per sentence on CPU, negligible
beside transcription. Methods, commands, the full table and limits are in
[benchmarks](../benchmarks.md), Measurement 4.

The Tatoeba numbers are optimistic because those sentences are very likely in
the model's training data. The Wikimedia numbers are the fairer signal. No
language has a grade yet and no human has read the output.

Measured the hard way: int8 quantisation makes the Spanish to English package
loop forever, so the safe setting is float32. That costs speed the CPU profile
was counting on, and any runtime choice has to be tested package by package
rather than assumed.

What closes it: a written review by a speaker per launch pair, numbers for the
remaining pairs, and a decision on whether the product ships these weights
in-process or keeps the existing server and command backends.

Through the product: the scores come from running the model directly, but the
shipped translation stage was exercised once end to end. `benchmarks/ollama_shim.py`
answers the local server protocol with the same engine, and a real run of
`dubbit dub clip.mp4 --from en --to pt --translator ollama` produced translated
SRT and VTT files. The command backend still takes no arguments, so a Python
script needs an executable wrapper on Windows; that path remains unexercised.

## S7 Voice synthesis in Rust

Question: is there a voice engine with a permissive licence that reaches
acceptable quality on CPU?

State: not started. Voice synthesis is out of scope until the dubbing milestone.

What closes it: at least one voice per launch language, latency and memory
measured on the light profile, and the weights documented in the model licences.

## S8 Background separation

Question: can Dubbit separate speech from background music or noise with weights
it is allowed to ship, on CPU, fast enough to sit beside transcription?

State: surveyed, not measured. The licence boundary decides more than the
benchmarks do, and it removes the obvious choices.

| Candidate | Task | Weights | Licence of the weights | Verdict |
| --- | --- | --- | --- | --- |
| GTCRN | Speech enhancement, not source separation | 523 KB ONNX | MIT, confirmed by the author for the pretrained weights | Accepted as the only cleared candidate |
| LavaSR | Speech restoration, optional denoising | About 50 MB | Apache-2.0 upstream, ONNX artefacts unclear | Needs its own export from the upstream model |
| Spleeter | Singing vocals against music | 19.7 MB per stem FP16 | Weights never explicitly granted; requests unanswered | Excluded until the maintainer answers |
| Open-Unmix UMX-HQ | Vocals against music | 35.6 MB | MIT on the weight record, but trained on a non-commercial set | Held back |
| Demucs | Vocals against music | 166 MB FP16 | Maintainer states the weights are for scientific use only | Excluded |
| UVR and MDX-Net | Vocals against music | 28 to 64 MB | No per-checkpoint licence | Excluded |

The open-source route: our code is Apache-2.0 and the product is free, so a
model that forbids commercial use is not automatically out. It may ship as an
optional, user-requested download, shown with its own licence and never as the
default, exactly as the audit already allows for non-commercial weights. That
reopens the music separators as optional downloads; it does not reopen them as
bundled defaults, and it does not make their licences compatible.

What this means: for a noisy interview, GTCRN is the only model that can be
shipped today, and it cleans speech rather than separating a music bed. For
dubbing, where the music has to be kept and the voice replaced, nothing on the
list is both cleared and measured. That is a real constraint on the dubbing
promise, not a detail.

What closes it: a measured run of GTCRN on the speech corpus (word error rate
before and after enhancement, time per minute of audio), and a licence answer
from one music separator vendor before any vocal-removal feature is promised.

## How a spike result is recorded

Each closed spike adds a section with the date, the exact command, the raw
output, the machine and the decision. A decision that changes the architecture
also gets an entry in `docs/adr/`.
