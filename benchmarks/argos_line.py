#!/usr/bin/env python3
import os
import sys

from argos_engine import translator


def main() -> int:
    source = os.environ.get("DUBBIT_SOURCE", "").strip() or "en"
    target = os.environ.get("DUBBIT_TARGET", "").strip()
    if not target:
        print("DUBBIT_TARGET is not set", file=sys.stderr)
        return 2
    text = sys.stdin.read()
    if not text.strip():
        return 0
    sys.stdout.write(translator(source, target)(text))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
