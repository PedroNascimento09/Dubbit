#!/usr/bin/env python3
import json
import pathlib
import subprocess
import sys
import uuid

SPDX = {
    "MIT": "MIT",
    "Apache-2.0": "Apache-2.0",
    "BSD-3-Clause": "BSD-3-Clause",
    "BSD-2-Clause": "BSD-2-Clause",
    "ISC": "ISC",
    "Unicode-3.0": "Unicode-3.0",
}


def metadata(root: pathlib.Path) -> dict:
    result = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--all-features"],
        cwd=root,
        check=True,
        capture_output=True,
    )
    return json.loads(result.stdout.decode("utf-8"))


def license_of(package: dict) -> str:
    value = package.get("license") or ""
    for ident in SPDX:
        if value.startswith(ident):
            return SPDX[ident]
    return "NOASSERTION"


def component(package: dict, root: bool) -> dict:
    entry = {
        "type": "application" if root else "library",
        "name": package["name"],
        "version": package["version"],
        "purl": f"pkg:cargo/{package['name']}@{package['version']}",
        "bom-ref": f"pkg:cargo/{package['name']}@{package['version']}",
        "licenses": [{"license": {"id": license_of(package)}}],
    }
    if package.get("repository"):
        entry["externalReferences"] = [{"type": "vcs", "url": package["repository"]}]
    return entry


def main() -> int:
    root = pathlib.Path(__file__).resolve().parent.parent
    target = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else root / "target" / "dubbit-sbom.cdx.json"
    data = metadata(root)
    workspace = set(data["workspace_members"])
    components = [
        component(package, package["id"] in workspace) for package in data["packages"]
    ]
    components.sort(key=lambda entry: entry["bom-ref"])
    document = {
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "serialNumber": f"urn:uuid:{uuid.uuid5(uuid.NAMESPACE_URL, 'dubbit-sbom')}",
        "version": 1,
        "metadata": {
            "tools": [{"vendor": "Dubbit", "name": "sbom.py", "version": "1"}],
            "component": {
                "type": "application",
                "name": "dubbit",
                "version": next(
                    package["version"]
                    for package in data["packages"]
                    if package["name"] == "dubbit-core"
                ),
            },
        },
        "components": components,
    }
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
    print(f"{target} ({len(components)} components)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
