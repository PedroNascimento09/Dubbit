#!/usr/bin/env python3
import hashlib
import pathlib
import sys


def digest(path: pathlib.Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: checksums.py FILE [FILE...]", file=sys.stderr)
        return 2
    for name in sys.argv[1:]:
        path = pathlib.Path(name)
        if not path.is_file():
            print(f"missing {path}", file=sys.stderr)
            return 1
        value = digest(path)
        side = path.with_name(path.name + ".sha256")
        side.write_text(f"{value}  {path.name}\n", encoding="utf-8")
        print(f"{value}  {path.name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
