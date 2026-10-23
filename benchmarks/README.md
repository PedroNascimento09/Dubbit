# Benchmarks

These scripts measure what Dubbit actually produces. They are development
tooling, not part of the distributed product, and they run offline once the
corpus exists.

## Files

| File | Purpose |
| --- | --- |
| `librispeech_corpus.py` | Samples chapters from an extracted LibriSpeech folder, converts clips to 16 kHz mono WAV with a short silence between them, and writes the concatenation list and the normalised reference transcript |
| `wer.py` | Reads a subtitle file and a reference transcript, normalises both, and prints word error rate with substitutions, deletions and insertions |

## Corpus

The published numbers use LibriSpeech `test-clean`, a public corpus by Vassil
Panayotov, licensed CC BY 4.0. The audio and the reference transcripts are not
in this repository; download them from `https://www.openslr.org/12` and extract
the archive. Attribution: LibriSpeech, CC BY 4.0.

The sample is deterministic: every sixteenth chapter, the first ten utterances
of each, giving 155 clips across different speakers. Each clip is converted to
16 kHz mono and padded with 0.35 s of silence, which keeps the concatenated
stream realistic and stops the recogniser from merging neighbours.

## Run

```sh
tar -xzf test-clean.tar.gz -C /data/librispeech
python benchmarks/librispeech_corpus.py \
  --root /data/librispeech/LibriSpeech/test-clean \
  --ffmpeg /path/to/ffmpeg \
  --out /data/corpus

ffmpeg -f concat -safe 0 -i /data/corpus/list.txt \
  -ar 16000 -ac 1 -c:a pcm_s16le /data/corpus/speech.wav

dubbit dub /data/corpus/speech.wav --from en --no-translate \
  --model whisper-tiny-q5_1 --output /data/run-tiny

python benchmarks/wer.py \
  --reference /data/corpus/reference.txt \
  --srt /data/run-tiny/subtitles/en.srt \
  --label whisper-tiny-q5_1
```

Repeat the last two commands for each model. Keep the run folder until the
numbers are written down; every claim in `docs/benchmarks.md` points at a run
made this way.

## Interpretation

- Text is lowercased, punctuation is removed and words are split on anything
  that is not a letter, digit or apostrophe on both sides, so casing and
  punctuation do not count as errors.
- Word error rate is `(substitutions + deletions + insertions) / reference words`.
- The hypothesis is the concatenation of all cue texts in the SRT, in order.
  Cue boundaries do not affect the result, which is why one long file can be
  measured in one run.
- Numbers from a different corpus, language or padding are not comparable.

## Adding a measurement

Add the corpus, the model, the profile, the command and the raw numbers to
`docs/benchmarks.md`. Do not edit an older entry; add a new one with a date. A
measurement without a command that produces it does not belong in the docs.
