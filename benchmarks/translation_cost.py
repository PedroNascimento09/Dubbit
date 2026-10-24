#!/usr/bin/env python3
import argparse
import json
import pathlib
import time

import psutil
from argostranslate import package

from argos_engine import translator

LINES = [
    "He hoped there would be stew for dinner.",
    "The train leaves at seven in the morning.",
    "She has been reading that book for a week.",
    "Please close the window before you go.",
    "They will arrive tomorrow afternoon.",
]


def size_on_disk(path: pathlib.Path) -> int:
    return sum(item.stat().st_size for item in path.rglob("*") if item.is_file())


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", default="en")
    parser.add_argument("--target", default="pt")
    options = parser.parse_args()
    process = psutil.Process()
    baseline = process.memory_info().rss
    started = time.perf_counter()
    engine = translator(options.source, options.target)
    first = engine(LINES[0])
    first_line = time.perf_counter() - started
    loaded = process.memory_info().rss
    started = time.perf_counter()
    for line in LINES:
        engine(line)
    per_line = (time.perf_counter() - started) / len(LINES)
    chosen = next(
        item
        for item in package.get_installed_packages()
        if item.from_code == options.source and item.to_code == options.target
    )
    print(
        json.dumps(
            {
                "pair": f"{options.source}-{options.target}",
                "package_version": chosen.package_version,
                "model_bytes": size_on_disk(chosen.package_path),
                "seconds_to_first_line": round(first_line, 2),
                "seconds_per_line": round(per_line, 3),
                "baseline_mb": round(baseline / (1024 * 1024), 1),
                "loaded_mb": round(loaded / (1024 * 1024), 1),
                "added_mb": round((loaded - baseline) / (1024 * 1024), 1),
                "first_line": first,
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
