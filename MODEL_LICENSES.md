# Model licences

Dubbit does not ship model weights in this repository. Each entry in
[models/whisper.json](models/whisper.json) pins a URL, a byte size and a SHA-256
checksum, and `dubbit models install` downloads the file only when the user asks
for it.

All four models below are quantised builds of Whisper weights published for the
whisper.cpp runtime. The weights are MIT licensed, which permits commercial use
and redistribution. The checksums in the manifest are the ones verified by
Dubbit before a model is used.

| Model id | File | Bytes | SHA-256 | Licence |
| --- | --- | --- | --- | --- |
| `whisper-tiny-q5_1` | `ggml-tiny-q5_1.bin` | 32 152 673 | `818710568da3ca15689e31a743197b520007872ff9576237bda97bd1b469c3d7` | MIT |
| `whisper-base-q5_1` | `ggml-base-q5_1.bin` | 59 707 625 | `422f1ae452ade6f30a004d7e5c6a43195e4433bc370bf23fac9cc591f01a8898` | MIT |
| `whisper-small-q5_1` | `ggml-small-q5_1.bin` | 190 085 487 | `ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb` | MIT |
| `whisper-large-v3-turbo-q5_0` | `ggml-large-v3-turbo-q5_0.bin` | 574 041 195 | `394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2` | MIT |

The authoritative hash for every model is the `sha256` field in the manifest. If
this page and the manifest disagree, the manifest wins and the page is fixed.

## Where the files come from

Each manifest entry points to a commit-pinned path in the public whisper.cpp
model repository, so the bytes behind a URL cannot change silently. A changed
file fails the checksum and is deleted rather than used.

## What this means for you

- The model licence is separate from the Dubbit licence. The Apache-2.0 licence
  of this project does not extend to the weights you download.
- The MIT licence of the weights allows commercial use. Check the upstream
  repository before redistributing a model yourself.
- Speech recognition output depends on the model. A larger model is slower and
  more accurate; Dubbit reports the memory estimate for each.
