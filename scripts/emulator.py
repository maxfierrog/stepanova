#!/usr/bin/env python3
"""Provision an rmemu image for a Paper Pro firmware version.

Usage: scripts/emulator.py RMEMU_DIR VERSION

Downloads the firmware from the mirrors in codexctl's version index (pinned to
a commit, checksum-verified, cached in RMEMU_DIR/firmware), then builds the
rmemu:VERSION Docker image so `rmemu start VERSION` can boot it. rmemu's own
setup command uses an older index that lacks recent firmware.
"""

import hashlib
import json
import os
import subprocess
import sys
import urllib.request

INDEX = (
    "https://raw.githubusercontent.com/Jayy001/codexctl/"
    "223a3236c7a0d7cdaf6c30400a39c9eb77f61e8b/data/version-ids.json"
)


def digest(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def fetch(emu, version):
    index = json.load(urllib.request.urlopen(INDEX))
    name, sha = index["remarkablepp"][version]
    dest = os.path.join(emu, "firmware", name)
    os.makedirs(os.path.dirname(dest), exist_ok=True)
    if os.path.exists(dest) and digest(dest) == sha:
        return dest
    mirrors = [u for u in index["external-provider-urls"] if "REPLACE_ID" in u]
    for mirror in mirrors:
        url = mirror.replace("REPLACE_ID", name)
        try:
            with urllib.request.urlopen(url, timeout=120) as r, open(dest, "wb") as f:
                while block := r.read(1 << 20):
                    f.write(block)
        except OSError as e:
            print(f"emulator: {url}: {e}", file=sys.stderr)
            continue
        if digest(dest) == sha:
            return dest
        print(f"emulator: {url}: checksum mismatch", file=sys.stderr)
    sys.exit(f"emulator: no mirror served firmware {version}")


def built(version):
    out = subprocess.run(
        ["docker", "images", "-q", f"rmemu:{version}"],
        capture_output=True, text=True, check=True,
    )
    return bool(out.stdout.strip())


def main():
    emu, version = sys.argv[1], sys.argv[2]
    if built(version):
        print(f"emulator: rmemu:{version} already built")
        return
    swu = fetch(emu, version)
    subprocess.run([os.path.join(emu, "tools", "import-rootfs.sh"), swu], check=True)
    subprocess.run(
        [
            "docker", "build", "-q",
            "--build-arg", f"ROOTFS=rmpp-rootfs:{version}",
            "-t", f"rmemu:{version}",
            "-f", os.path.join(emu, "docker", "Dockerfile"),
            emu,
        ],
        check=True,
    )
    print(f"emulator: built rmemu:{version}")


if __name__ == "__main__":
    main()
