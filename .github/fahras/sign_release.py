#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Signs the update manifest for a Fahras Windows release.

Writes strata-update-manifest.json and its signatures next to the installer,
in the format Fahras verifies before it runs an update. The private key comes
from the UPDATE_SIGNING_KEY environment variable.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
from sign_update_manifest import (  # noqa: E402
    MANIFEST_NAME,
    MAX_ARCHIVE_BYTES,
    SIGNATURES_NAME,
    VERSION,
    sign_manifest,
)

REPOSITORY = "iBar8/Fahras"
TARGET = "x86_64-pc-windows-gnu"


def manifest_bytes(directory, version, source_commit):
    if not VERSION.fullmatch(version):
        raise ValueError("Invalid release version")
    if not re.fullmatch(r"[0-9a-f]{40}", source_commit):
        raise ValueError("Invalid source commit")
    name = f"Fahras-Setup-{version}-x64.exe"
    data = (directory / name).read_bytes()
    if not 0 < len(data) <= MAX_ARCHIVE_BYTES:
        raise ValueError(f"Invalid installer size: {name}")
    artifact = {"name": name, "target": TARGET, "size": len(data), "sha256": hashlib.sha256(data).hexdigest()}
    value = {"schema": 1, "repository": REPOSITORY, "tag": f"v{version}", "source_commit": source_commit, "artifacts": [artifact]}
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode("utf-8")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--directory", type=Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--source-commit", required=True)
    args = parser.parse_args()
    pem = os.environ.pop("UPDATE_SIGNING_KEY", "")
    if not pem.strip():
        parser.error("UPDATE_SIGNING_KEY is empty; add the FAHRAS_UPDATE_SIGNING_KEY secret")
    try:
        manifest = manifest_bytes(args.directory, args.version, args.source_commit)
        trusted_keys = json.loads((ROOT / "data/update-keys.json").read_text())
        with tempfile.TemporaryDirectory(prefix="fahras-sign-") as staging:
            key = Path(staging) / "key.pem"
            with open(key, "x", opener=lambda name, flags: os.open(name, flags, 0o600)) as file:
                file.write(pem.strip() + "\n")
            signatures = sign_manifest(manifest, [key], trusted_keys)
        (args.directory / MANIFEST_NAME).write_bytes(manifest)
        (args.directory / SIGNATURES_NAME).write_bytes(signatures)
    except (ValueError, OSError) as error:
        parser.exit(1, f"Release signing failed: {error}\n")


if __name__ == "__main__":
    main()
