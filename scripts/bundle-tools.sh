#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tools="${1:-${DUBBIT_TOOLS_DIR:-$root/.tools/bin}}"
whisper="${2:-${DUBBIT_WHISPER_DIR:-$root/.tools/whisper/bin/Release}}"
target="$root/apps/desktop/src-tauri/resources/bin"

for binary in ffmpeg ffprobe; do
  source="$tools/$binary.exe"
  if [[ ! -f "$source" ]]; then
    source="$tools/$binary"
  fi
  if [[ ! -f "$source" ]]; then
    echo "missing $binary in $tools" >&2
    exit 1
  fi
  cp "$source" "$target/"
done

if [[ ! -f "$whisper/whisper-cli.exe" ]]; then
  echo "missing whisper-cli in $whisper" >&2
  exit 1
fi

mkdir -p "$target"
cp "$whisper/whisper-cli.exe" "$target/"
cp "$whisper"/*.dll "$target/"

echo "bundled tools in $target"
du -sh "$target"
