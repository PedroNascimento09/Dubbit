# Install

Dubbit runs on your machine and produces subtitles from video or audio. This page installs the current source build on Windows or Linux. macOS is not tested yet.

## Windows installer

The window ships with FFmpeg, FFprobe and the speech recogniser, so nothing else has to be installed by hand. There is no public release yet; build the installer from source with Node.js, Rust and the tools in `.tools/bin` and `.tools/whisper/bin/Release`:

```sh
bash scripts/bundle-tools.sh
cd apps/desktop
npm install
npm run tauri build
```

The installer is written to `target/release/bundle/nsis/` and measured about 97 MiB with the bundled tools. Running it installs for the current user in `%LOCALAPPDATA%\Dubbit Desktop`, without administrator rights.

Speech models are not inside the installer. The window downloads the model it needs the first time, with the size shown before it starts, and keeps it in `%LOCALAPPDATA%\Dubbit\models`.

Uninstalling removes the program and leaves the data folder, with your models and queue, in place.

The rest of this page covers the command line and building from source without the installer.

## What you need

- Rust, installed with `rustup`. The pinned toolchain in `rust-toolchain.toml` is selected automatically.
- FFmpeg and FFprobe.
- A `whisper-cli` speech recognition executable.
- Free disk space: the smallest model is about 31 MB, and a project folder keeps an uncompressed copy of the audio, so allow room for both.

Windows and Linux are the supported targets. The CPU path is the guaranteed path; a GPU is not required.

## 1. Get the code

```sh
git clone https://github.com/PedroNascimento09/Dubbit.git
cd Dubbit
```

## 2. Build

```sh
cargo build --release
```

The binary is `target/release/dubbit.exe` on Windows and `target/release/dubbit` on Linux. Add it to your `PATH`, or call it with its full path.

## 3. Make FFmpeg available

Put `ffmpeg` and `ffprobe` on your `PATH`. If you keep them somewhere else, point Dubbit at that folder:

```sh
export DUBBIT_TOOLS_DIR="$HOME/tools/bin"
```

PowerShell:

```powershell
$env:DUBBIT_TOOLS_DIR = "C:\tools\bin"
```

Individual executables can be set instead:

```sh
export DUBBIT_FFMPEG=/usr/local/bin/ffmpeg
export DUBBIT_FFPROBE=/usr/local/bin/ffprobe
```

## 4. Make speech recognition available

Dubbit calls an executable named `whisper-cli`. Either place it on your `PATH` or name it directly:

```sh
export DUBBIT_ASR=/opt/speech/whisper-cli
```

PowerShell:

```powershell
$env:DUBBIT_ASR = "C:\tools\speech\whisper-cli.exe"
```

On Windows the executable may need its companion `.dll` files in the same folder.

## 5. Check the machine

```sh
dubbit doctor
```

The report shows the operating system, memory, processors, detected graphics adapters, the chosen profile, the media tools that were found and the state of each model. It exits successfully even when something is missing, so read the output rather than the exit code.

To keep a copy for a bug report:

```sh
dubbit doctor --report report.json
```

Review the file before sharing it.

## 6. Download a model

Models are downloaded only when you ask for one. Start with the smallest:

```sh
dubbit models install whisper-tiny-q5_1
```

Downloads resume after an interruption and the SHA-256 checksum is verified before the file is used. See what is available and what is already on disk:

```sh
dubbit models list
```

## 7. First run

```sh
dubbit dub interview.mp4 --from en --to pt
```

The subtitles appear in `interview.dubbit/subtitles/`. To transcribe without translating:

```sh
dubbit dub interview.mp4 --no-translate
```

## Where things are stored

Dubbit keeps its own files in a data folder:

- Windows: `%LOCALAPPDATA%\Dubbit`
- Linux: `$XDG_DATA_HOME/dubbit`, usually `~/.local/share/dubbit`
- macOS: `~/Library/Application Support/Dubbit`

The folder holds `config.json`, `state.json` and `models/`. Set `DUBBIT_HOME` to move it:

```sh
export DUBBIT_HOME=/mnt/fast/dubbit
```

Your media and the project folders stay wherever you point Dubbit. Nothing is uploaded.

## Change settings

```sh
dubbit config set profile light
dubbit config set translator command
dubbit config show
dubbit config path
```

## Uninstall

Uninstall Dubbit Desktop from Windows settings, or delete the binary when it was built from source. Then delete the data folder and the project folders you created. No service, registry key or startup entry is left behind.

## Before you file a bug

Run `dubbit doctor --report report.json`, include the error code shown by the failed command, and say what you expected. Do not attach private media.
