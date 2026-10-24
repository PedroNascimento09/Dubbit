#!/usr/bin/env python3
import hashlib
import json
import pathlib
import platform
import shutil
import sys
import tarfile
import urllib.request
import zipfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "models" / "tools.json"
CACHE = ROOT / ".tools" / "cache"
STAGE = ROOT / ".tools" / "tools"
TARGET = ROOT / "apps" / "desktop" / "src-tauri" / "resources" / "bin"


def current_platform() -> str:
    system = platform.system()
    machine = platform.machine().lower()
    arch = "arm64" if machine in {"arm64", "aarch64"} else "x64"
    if system == "Windows":
        return f"windows-{arch}"
    if system == "Linux":
        return f"linux-{arch}"
    if system == "Darwin":
        return f"macos-{arch}"
    raise SystemExit(f"unsupported system {system}")


def digest(path: pathlib.Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def fetch(entry: dict) -> pathlib.Path:
    CACHE.mkdir(parents=True, exist_ok=True)
    name = entry["url"].rsplit("/", 1)[-1]
    target = CACHE / name
    if target.is_file():
        if target.stat().st_size == entry["size"] and digest(target) == entry["sha256"]:
            return target
        target.unlink()
    partial = target.with_suffix(target.suffix + ".part")
    offset = partial.stat().st_size if partial.is_file() else 0
    request = urllib.request.Request(entry["url"])
    if offset:
        request.add_header("Range", f"bytes={offset}-")
    print(f"downloading {name} ({entry['size']} bytes)", flush=True)
    with urllib.request.urlopen(request) as response:
        mode = "ab" if offset and response.status == 206 else "wb"
        with partial.open(mode) as handle:
            shutil.copyfileobj(response, handle, 1024 * 1024)
    if partial.stat().st_size != entry["size"]:
        raise SystemExit(f"size mismatch for {name}")
    if digest(partial) != entry["sha256"]:
        raise SystemExit(f"sha256 mismatch for {name}")
    partial.replace(target)
    return target


def unpack(archive: pathlib.Path, destination: pathlib.Path, entry: dict) -> None:
    if destination.is_dir():
        shutil.rmtree(destination)
    destination.mkdir(parents=True)
    if entry["format"] == "zip":
        with zipfile.ZipFile(archive) as bundle:
            bundle.extractall(destination)
        return
    with tarfile.open(archive) as bundle:
        bundle.extractall(destination, filter="data")


def stage(tool: dict, entry: dict, archive: pathlib.Path) -> pathlib.Path:
    platform_root = STAGE / entry["platform"]
    extracted = STAGE / "archive" / entry["platform"] / tool["name"]
    unpack(archive, extracted, entry)
    source = extracted / entry["prefix"] if entry["prefix"] else extracted
    resolved = sorted({path for pattern in entry["binaries"] for path in source.glob(pattern)})
    if not resolved:
        raise SystemExit(f"no binaries matched for {tool['name']} on {entry['platform']}")
    ready = platform_root / tool["name"]
    if ready.is_dir():
        shutil.rmtree(ready)
    ready.mkdir(parents=True)
    for path in resolved:
        (ready / path.name).write_bytes(path.read_bytes())
        (ready / path.name).chmod(0o755)
    return ready


def main() -> int:
    wanted = sys.argv[1] if len(sys.argv) > 1 else current_platform()
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    if TARGET.is_dir():
        shutil.rmtree(TARGET)
    TARGET.mkdir(parents=True)
    total = 0
    for tool in manifest["tools"]:
        entry = next((item for item in tool["files"] if item["platform"] == wanted), None)
        if entry is None:
            raise SystemExit(f"{tool['name']} has no build for {wanted}")
        archive = fetch(entry)
        ready = stage(tool, entry, archive)
        for path in sorted(ready.iterdir()):
            destination = TARGET / path.name
            shutil.copy2(path, destination)
            total += destination.stat().st_size
            print(f"  {tool['name']} {tool['version']} {path.name}")
    print(f"bundled {len(list(TARGET.iterdir()))} files, {total // (1024 * 1024)} MiB in {TARGET}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
