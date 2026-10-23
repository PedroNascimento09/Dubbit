# Troubleshooting

Every failure prints a code, what happened and a next step. Start with the code, then read the section below.

Run this first when something looks wrong:

```sh
dubbit doctor
```

## Codes

| Code | Meaning |
| --- | --- |
| DUB-0001 | An option or setting value is not valid |
| DUB-0002 | The run was cancelled |
| DUB-1001 | FFmpeg or FFprobe was not found |
| DUB-1002 | The media file cannot be read |
| DUB-1003 | The media has no usable audio or video |
| DUB-1004 | FFmpeg or FFprobe failed on the file |
| DUB-1101 | A needed component is missing |
| DUB-1102 | A needed component failed |
| DUB-2001 | The language code is not supported |
| DUB-2101 | No speech was detected |
| DUB-3001 | The project was made by a newer version |
| DUB-3002 | The project file is not valid |
| DUB-3003 | The project folder already exists |
| DUB-3004 | A file or folder could not be written |
| DUB-3101 | A model download failed |
| DUB-3102 | A downloaded file is damaged |
| DUB-3103 | The model id is not in the manifest |
| DUB-3104 | The model is not installed |

## DUB-1001: media tools not found

Dubbit needs `ffmpeg` and `ffprobe`. Put both on your `PATH`, or set:

```sh
export DUBBIT_TOOLS_DIR=/path/to/bin
```

On Windows use PowerShell:

```powershell
$env:DUBBIT_TOOLS_DIR = "C:\tools\bin"
```

Check with `dubbit doctor`; both lines must show a version and a location.

## DUB-1002 and DUB-1003: the file cannot be read

- Confirm the path is right and the file is complete, not a partial download.
- Play the file in a normal player. If it does not play there, Dubbit cannot read it either.
- Folders and file names with spaces, accents or long paths are supported; quote the path in the shell.
- A file with a video stream and a separate audio stream in another container needs remuxing first:

```sh
ffmpeg -i input.mkv -c copy output.mp4
```

## DUB-1004: the media tool failed

FFmpeg read the file but stopped. The message includes its last line. Common causes are a damaged file, an unsupported codec, or a network path that disconnected. Copy the file to a local disk and try again. Include the full message when opening an issue.

## DUB-1101 and DUB-1102: the recogniser

Dubbit runs an executable named `whisper-cli`. On Windows the companion `.dll` files must sit next to it.

```sh
export DUBBIT_ASR=/path/to/whisper-cli
dubbit doctor
```

If the recogniser starts but fails, try the smallest model to separate a model problem from a file problem:

```sh
dubbit dub clip.mp4 --model whisper-tiny-q5_1 --no-translate
```

## DUB-2001: unsupported language

```sh
dubbit languages
```

Use a code from that list, for example `en`, `pt`, `pt-BR` or `es`. A region is normalised; an unknown base code is rejected.

## DUB-2101: no speech detected

The audio track holds nothing the recogniser can use.

- Check the track is not silent or muted in the container.
- Set the language explicitly with `--from`.
- Try a larger model, which is more robust with quiet or noisy speech:

```sh
dubbit models install whisper-base-q5_1
dubbit dub clip.mp4 --model whisper-base-q5_1
```

Music, applause and room tone alone produce this code.

## DUB-3003: the project folder exists

Dubbit never overwrites a finished or interrupted project without being asked.

- Continue it: add `--resume`.
- Or write somewhere else: add `--output other-folder`.

`--resume` checks the checksum of the source media. If the media changed, the run stops instead of mixing old and new text.

## DUB-3001, DUB-3002: project files

DUB-3001 means the project was written by a newer Dubbit than the one you are running; update the binary. DUB-3002 means `project.json` is unreadable or a segment checkpoint is missing. Keep the original media and start a fresh project; the media is never modified.

## DUB-3004: cannot write

- Check the folder exists and you can write to it.
- Check free disk space. A one hour video needs several hundred megabytes for audio and checkpoints.
- Avoid writing into a folder that a sync client is locking.

## DUB-3101 and DUB-3102: downloads

Downloads resume automatically; run the same command again.

```sh
dubbit models install whisper-small-q5_1
```

If it fails again:

- Check the connection and any proxy that inspects TLS traffic.
- Confirm the system certificate store is readable; Dubbit uses the operating system trust store.
- If the checksum still fails, the partial file is deleted and fetched again on the next run.

Verify what is on disk:

```sh
dubbit models verify
```

## DUB-3103 and DUB-3104: models

```sh
dubbit models list
```

DUB-3103 means the id is not in the manifest. DUB-3104 means the id is valid but the file is not downloaded. Install it, or let the profile pick a model you already have:

```sh
dubbit dub clip.mp4 --model whisper-tiny-q5_1
```

## It is slow

The recogniser uses the CPU in this build. Useful checks:

- `dubbit doctor` shows the chosen profile and thread count.
- `--profile light` and the smallest model are the fastest combination.
- `--threads` above the number of physical cores rarely helps.
- Very long files take proportionally longer; process a shorter clip first to confirm the setup.

Report timings with the model, the processor and the length of the media. Without those three numbers a timing report cannot be compared.

## Subtitles look wrong

- Wrong language: set `--from` explicitly.
- Translation left the text unchanged: the `auto` backend found no local model server and passed text through. Choose a backend or configure one; see the user guide.
- Names translated inconsistently: add them to a glossary CSV.
- Lines feel long: cues are limited to 42 characters per line and two lines per cue; a single very long word is not broken.

## Getting help

Open an issue with:

- the `dubbit --version` output;
- the operating system and processor;
- the command you ran;
- the full error output, including the code;
- the report from `dubbit doctor --report report.json`, after removing anything private.

Never attach private media, transcripts or personal paths.
