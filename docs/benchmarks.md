# Benchmarks

Dubbit publishes only measurements it made itself, with the command used to make
them. Four measurements exist: a short clip that shows start-up cost, a long
file that shows throughput under a hard memory cap, word error rate on public
speech, and translation quality on public sentence pairs.

## How to reproduce a transcription measurement

```sh
cargo build --release
./target/release/dubbit models install whisper-tiny-q5_1
time ./target/release/dubbit dub samples/clip.wav --from en --no-translate
```

`--no-translate` isolates transcription. Report the media duration, the wall
time, the model, the profile, the operating system, the processor, the memory
limit and the peak memory.

On Windows the measurement can be repeated inside a job object with a memory
cap, which is how the numbers below were produced. The wall time covers the
whole command: process start, audio extraction, transcription and subtitle
writing.

## Measurement 1: short clip

| Field | Value |
| --- | --- |
| Date | 2026-10-23 |
| Media | 7.36 s of clean English speech, 16-bit PCM WAV, mono |
| Model | `whisper-tiny-q5_1` |
| Profile | light, auto-selected |
| Build | release |
| System | Windows 11, 12 logical cores, 15 GB of RAM |
| Graphics | Discrete adapter detected, not used for inference |
| Wall time | 1.86 s, 1.70 s and 1.70 s over three runs |
| Ratio | about 4x faster than real time |

At this length the run is dominated by process start and model load, so the
ratio understates throughput. Measurement 2 is the useful one.

## Measurement 2: long file under a hard 8 GiB cap

| Field | Value |
| --- | --- |
| Date | 2026-10-23 |
| Media | 817.26 s (13 min 37 s) of continuous synthesised English speech, WAV, 36 MB, mono |
| System | Windows 11, 12 logical cores, 15 GB of RAM |
| Graphics | Discrete adapter detected, not used for inference |
| Build | release |
| Memory limit | 8 GiB, enforced by a Windows job object on the process and the job, including the recognition child process |
| Network | no request is made in the processing code path; not instrumented at the socket level in this run |
| Translation | disabled |

| Model | Profile | Wall time | Ratio | Peak job memory |
| --- | --- | --- | --- | --- |
| `whisper-tiny-q5_1` | light | 47.64 s | 17.2x real time | 731.5 MB |
| `whisper-base-q5_1` | light, model forced | 69.79 s | 11.7x real time | 789.2 MB |
| `whisper-small-q5_1` | balanced | 200.44 s | 4.1x real time | 1055.0 MB |

All three runs completed with exit code 0 and produced 223 to 224 subtitle cues
covering the whole file, from the first sentence to the last.

The cap never bound: peak use stayed between 0.7 GB and 1.1 GB. That establishes
that these models fit comfortably inside an 8 GB machine with the operating
system and a browser running, but it does not exercise an out-of-memory path.

### What this measurement does not say

- The speech was synthesised from one voice, with no music, noise, crosstalk or
  accent variation. Real media is harder for the recogniser and for timing.
- Accuracy is in Measurement 3, not in this timing run.
- One machine, one operating system. Linux, macOS and smaller processors are
  unmeasured.
- Ratio falls quickly with model size. A 13 minute file on the small model still
  takes over three minutes, so long recordings need the light or base model on
  modest hardware.

## Measurement 3: word error rate

| Field | Value |
| --- | --- |
| Date | 2026-10-23 |
| Corpus | LibriSpeech `test-clean`, CC BY 4.0, sampled deterministically: 16 chapters, first 10 utterances of each, 155 clips |
| Reference | 3429 normalised words |
| Audio | 1425.07 s (23 min 45 s), 16 kHz mono, 0.35 s of silence between clips |
| System | Windows 11, 12 logical cores, 15 GB of RAM |
| Build | release, memory capped at 8 GiB |
| Language | English, forced with `--from en` |
| Decoding | greedy, the recogniser default |

| Model | Profile | WER | Errors | Substitutions | Deletions | Insertions |
| --- | --- | --- | --- | --- | --- | --- |
| `whisper-tiny-q5_1` | light | 8.22% | 282 | 242 | 17 | 23 |
| `whisper-base-q5_1` | light, model forced | 5.25% | 180 | 154 | 21 | 5 |
| `whisper-small-q5_1` | balanced | 4.52% | 155 | 128 | 20 | 7 |

The ordering is the expected one: each larger model is more accurate. The tool
that produced these numbers is in [benchmarks/](../benchmarks/README.md), with
the corpus build and the exact commands.

### What this measurement does not say

- Only English, only clean read speech from the public corpus. Accents, noise,
  music and crosstalk are untested.
- Punctuation and casing are normalised away by the scorer. Spoken numbers must
  still match the reference words.
- Scores are on a concatenated stream of separate clips, not a real recording
  with natural turn taking.
- No other model family and no other decoding setting was scored. Translation
  quality is measured separately, in Measurement 4.

## Measurement 4: translation quality

| Field | Value |
| --- | --- |
| Date | 2026-10-24 |
| Engine | OPUS-MT weights as packaged by Argos Translate 1.9, run through CTranslate2 4.x on CPU, int8, default decoding |
| Corpus 1 | Tatoeba en-pt, version 2023-04-12, CC BY 2.0 FR, 200 sentence pairs sampled with seed 2026 |
| Corpus 2 | Wikimedia en-pt, version 20230407, CC BY-SA, 200 sentence pairs, same filter and seed |
| Filter | both sides between 20 and 200 characters, duplicate sources removed |
| Metric | chrF and BLEU from SacreBLEU 2.6, defaults |
| System | Windows 11, 12 logical cores, 15 GB of RAM, no GPU |

| Pair | Corpus | chrF | BLEU | Segments | Source words | Seconds |
| --- | --- | --- | --- | --- | --- | --- |
| en → pt | Tatoeba | 68.45 | 49.18 | 200 | 1399 | 11.3 |
| pt → en | Tatoeba | 76.87 | 65.02 | 200 | 1500 | 12.3 |
| en → pt | Wikimedia | 71.06 | 52.49 | 200 | 3549 | 37.6 |
| pt → en | Wikimedia | 71.67 | 52.36 | 200 | 3355 | 34.4 |

Cost is 0.06 seconds per short sentence and 0.17 to 0.19 seconds per longer
one, which is two to three orders of magnitude below the transcription stage.
Translation is not a bottleneck.

The commands:

```sh
python benchmarks/translation_quality.py --corpus tatoeba --source en --target pt --limit 200
python benchmarks/translation_quality.py --corpus tatoeba --source pt --target en --limit 200
python benchmarks/translation_quality.py --corpus wikimedia --source en --target pt --limit 200
python benchmarks/translation_quality.py --corpus wikimedia --source pt --target en --limit 200
```

### What this measurement does not say

- These are automatic metrics, not reader judgement. Nobody who speaks the pair
  has read the output yet. A chrF near 70 hides flat sentences, wrong register
  and mistranslated names; the plan still requires a written review per pair.
- Tatoeba is short, simple and very likely inside the model's training data, so
  the two Tatoeba rows are optimistic. The Wikimedia rows are the more honest
  signal for real text.
- Only English and Portuguese. Spanish, French and German have no number, and
  neither does any pair reached through a pivot language.
- Nothing was measured through the product's own pipeline. The benchmark runs
  the model directly, because the shipped path offers a local server or an
  external command, and this machine has neither wired to a model. Subtitle
  segment boundaries, glossary protection and cue splitting are outside the
  measurement.
- Tatoeba and Wikimedia sentences are already clean text. Speech that has been
  transcribed with errors will translate worse, and that compounding is not
  measured here.

## Planned measurements

The roadmap calls for a wider suite: noisy and multi-speaker audio, other
languages, a translation number for every language pair that would be
advertised, a reader review of translated output, and the largest model. Results will be added here with the raw numbers and the exact commands,
and previous entries will be kept rather than edited.

## Rules

- No number appears here without a command that produced it.
- A run that failed is reported as a failure, not removed.
- No comparison with another product appears in a benchmark table.
