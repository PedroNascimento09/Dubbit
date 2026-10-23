import argparse
import pathlib
import subprocess
import sys

from wer import normalize


def select_chapters(root, chapter_count):
    chapters = sorted(root.glob("*/*/*.trans.txt"))
    if not chapters:
        raise SystemExit(f"no transcript files under {root}")
    step = max(1, len(chapters) // chapter_count)
    return chapters[::step][:chapter_count]


def read_chapter(chapter, per_chapter):
    entries = []
    rows = chapter.read_text(encoding="utf-8").strip().splitlines()
    for row in rows[:per_chapter]:
        identifier, text = row.split(" ", 1)
        audio = chapter.parent / f"{identifier}.flac"
        if audio.is_file():
            entries.append((audio, text))
    return entries


def convert(ffmpeg, source, target, pad):
    if target.is_file():
        return
    subprocess.run(
        [
            ffmpeg,
            "-nostdin",
            "-y",
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            str(source),
            "-af",
            f"apad=pad_dur={pad}",
            "-ar",
            "16000",
            "-ac",
            "1",
            "-c:a",
            "pcm_s16le",
            str(target),
        ],
        check=True,
    )


def main():
    parser = argparse.ArgumentParser(
        description="Build a concatenated speech corpus and reference transcript from LibriSpeech."
    )
    parser.add_argument("--root", required=True, help="Extracted LibriSpeech test-clean folder")
    parser.add_argument("--ffmpeg", required=True, help="Path to the FFmpeg executable")
    parser.add_argument("--out", required=True, help="Folder for clips, list and reference")
    parser.add_argument("--chapters", type=int, default=16, help="Chapters to sample")
    parser.add_argument("--utterances", type=int, default=10, help="Utterances per chapter")
    parser.add_argument("--pad", type=float, default=0.35, help="Silence after each clip in seconds")
    arguments = parser.parse_args()

    root = pathlib.Path(arguments.root)
    output = pathlib.Path(arguments.out)
    clips = output / "clips"
    clips.mkdir(parents=True, exist_ok=True)

    utterances = []
    for chapter in select_chapters(root, arguments.chapters):
        utterances.extend(read_chapter(chapter, arguments.utterances))

    concat = []
    reference = []
    for index, (audio, text) in enumerate(utterances):
        target = clips / f"{index:04d}.wav"
        convert(arguments.ffmpeg, audio, target, arguments.pad)
        concat.append(f"file '{target.resolve().as_posix()}'")
        reference.extend(normalize(text))

    (output / "list.txt").write_text("\n".join(concat) + "\n", encoding="utf-8")
    (output / "reference.txt").write_text(" ".join(reference) + "\n", encoding="utf-8")
    print(
        f"{len(utterances)} utterances, {len(reference)} reference words, clips in {clips}"
    )


if __name__ == "__main__":
    sys.exit(main())
