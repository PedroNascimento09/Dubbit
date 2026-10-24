# Desktop application

The desktop window is the normal way to use Dubbit. It wraps the same engine as the command line: same project folders, same checkpoints, same subtitle files.

Status: development preview. A Windows installer can be built from source and installs the window with FFmpeg, FFprobe and the speech recogniser included. There is no public release yet.

## What the window does

1. **Add files.** Drop videos or audio onto the rail, or click the drop area to choose them. Several files can be queued at once.
2. **Choose the language.** Speech is detected automatically unless a language is chosen. The second list picks the subtitle language.
3. **Press Create subtitles.** One job runs at a time. The stage list shows which step is running, and every finished stage is written to disk, so a crash or a closed window never loses the whole run.
4. **Review.** When a run finishes, the editor opens. Each line has its time range, the recognised text, and the translated text when a translator is configured. Press the play button to hear that line, use the search box to find a phrase, then press Save and export.
5. **Export.** Subtitles are written as SRT and VTT in the project folder, one pair per language. Open folder shows them in the file manager.

## Translation

The window translates only when a translator is configured on the computer: a local model server such as Ollama, or a command configured in `config.json`. Without one, the subtitle language list is disabled and Dubbit writes subtitles in the spoken language only. It never fills a translation file with the original text and calls it translated.

Translation quality is not measured yet, so treat translated lines as a first draft to review.

## Where the files go

Each run writes a folder next to the source file:

```text
clip.dubbit/
  project.json
  audio.wav
  segments-source.json
  subtitles/en.srt
  subtitles/en.vtt
```

The queue itself lives in the application data folder, so a pending or interrupted job is still listed after a restart. Interrupted jobs restart from their last finished stage; jobs that were already done keep their results.

## Shortcuts

| Key | Action |
| --- | --- |
| Enter or Space on a queue row | Open that job |
| Ctrl+S or Command+S in the editor | Save and export |
| Tab | Move between the queue, the search box and the lines |

## Appearance

The window follows the system light or dark appearance and can be switched by hand with the button at the bottom of the rail. The choice is remembered. Motion is reduced automatically when the system asks for it.

## Build and run from source

Prerequisites are Rust, Node.js and Python. The build fetches FFmpeg, FFprobe and `whisper-cli` itself from the pinned manifest.

```sh
python scripts/bundle-tools.py
cd apps/desktop
npm install
npm run tauri build
```

The build embeds the interface and the media tools. It leaves the installer in `target/release/bundle/nsis/` and a runnable binary in `target/release`. The installed window needs no environment variables. A bare binary works when the tools sit in a `bin` folder beside it, or when the variables point at them:

```sh
export DUBBIT_TOOLS_DIR=/path/to/tools
export DUBBIT_ASR=/path/to/whisper-cli
./target/release/dubbit-desktop
```

For interface work, `npm run dev` serves the window in any browser with deterministic sample data, so layout and behaviour can be checked without building the shell. That preview never writes files.

The full set of checks is in [DEVELOPMENT.md](DEVELOPMENT.md).

## Limits

- One job at a time, first in first out. There is no priority control yet.
- Stop takes effect at the end of the current stage, because the recogniser runs as a separate process.
- The window does not bundle FFmpeg or the recogniser yet; a missing tool is reported with its name before a job starts.
- No dubbing, voice synthesis or lip sync. Subtitles only.
- Windows and Linux are the supported platforms. macOS is untested.
