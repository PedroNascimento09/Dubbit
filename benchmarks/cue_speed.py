#!/usr/bin/env python3
import argparse
import pathlib
import statistics

FAST = 17.0
TOO_FAST = 21.0


def seconds(stamp: str) -> float:
    hours, minutes, rest = stamp.split(":")
    whole, millis = rest.split(",")
    return int(hours) * 3600 + int(minutes) * 60 + int(whole) + int(millis) / 1000


def cues(path: pathlib.Path, limit: int | None) -> list[tuple[float, int]]:
    blocks = path.read_text(encoding="utf-8").strip().split("\n\n")
    rows: list[tuple[float, int]] = []
    for block in blocks:
        lines = [line for line in block.splitlines() if line.strip()]
        if len(lines) < 3:
            continue
        timing = lines[1]
        if "-->" not in timing:
            continue
        start, end = (part.strip() for part in timing.split("-->"))
        duration = seconds(end) - seconds(start)
        text = " ".join(lines[2:]).strip()
        if duration <= 0 or not text:
            continue
        rows.append((duration, len(text)))
        if limit is not None and len(rows) >= limit:
            break
    return rows


def percentile(values: list[float], share: float) -> float:
    ordered = sorted(values)
    index = max(0, min(len(ordered) - 1, int(share * len(ordered) + 0.999999) - 1))
    return ordered[index]


def measure(path: pathlib.Path, limit: int | None) -> dict:
    rows = cues(path, limit)
    if not rows:
        return {"file": path.name, "cues": 0}
    speeds = [characters / duration for duration, characters in rows]
    return {
        "file": path.name,
        "cues": len(rows),
        "characters": sum(characters for _, characters in rows),
        "seconds": round(sum(duration for duration, _ in rows), 1),
        "median": round(statistics.median(speeds), 1),
        "p90": round(percentile(speeds, 0.9), 1),
        "over_fast": sum(1 for speed in speeds if speed > FAST),
        "over_too_fast": sum(1 for speed in speeds if speed > TOO_FAST),
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Report subtitle reading speed: characters per second per cue."
    )
    parser.add_argument("--srt", nargs="+", required=True)
    parser.add_argument("--limit", type=int, default=None)
    options = parser.parse_args()
    header = f"{'file':<32}{'cues':>6}{'chars':>8}{'secs':>8}{'median':>8}{'p90':>7}{'>17':>6}{'>21':>6}"
    print(header)
    for name in options.srt:
        result = measure(pathlib.Path(name), options.limit)
        if not result["cues"]:
            print(f"{result['file']:<32}{'0':>6}  no cues")
            continue
        print(
            f"{result['file']:<32}{result['cues']:>6}{result['characters']:>8}"
            f"{result['seconds']:>8.1f}{result['median']:>8.1f}{result['p90']:>7.1f}"
            f"{result['over_fast']:>6}{result['over_too_fast']:>6}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
