#!/usr/bin/env python3
"""Fetch pinned originals without executing them; verify parent before slicing."""
import hashlib, sys, tomllib, urllib.parse, urllib.request
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
def config():
    return tomllib.loads((ROOT / "target.toml").read_text())
def fetch(spec):
    url = config()["r2_public"].rstrip("/") + "/" + urllib.parse.quote(spec["key"])
    req = urllib.request.Request(url, headers={"User-Agent": "t5-press-kit-analysis"})
    with urllib.request.urlopen(req, timeout=120) as r:
        data = r.read(spec["size"] + 1)
    if len(data) != spec["size"] or hashlib.sha256(data).hexdigest() != spec["sha256"]:
        raise ValueError("size or SHA-256 mismatch: " + spec["key"])
    return data
def main():
    ident, dest = sys.argv[1], Path(sys.argv[2])
    cfg = config()
    dest.mkdir(parents=True, exist_ok=True)
    for b in cfg["build"]:
        if b["id"] != ident: continue
        spec = b["exe"]
        data = fetch(spec)
        if "slice_offset" in spec:
            start, size = spec["slice_offset"], spec["slice_size"]
            if start < 0 or size <= 0 or start + size > len(data): raise ValueError("invalid slice")
            data = data[start:start+size]
        out = dest / (ident + ".exe")
        out.write_bytes(data)
        print(out, hashlib.sha256(data).hexdigest())
        return
    for s in cfg["swf"]:
        if s["id"] == ident:
            (dest / (ident + ".swf")).write_bytes(fetch(s))
            return
    raise ValueError("unknown build: " + ident)
if __name__ == "__main__": main()
