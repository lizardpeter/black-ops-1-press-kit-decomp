#!/usr/bin/env python3
"""Check Windows embedded movie against independently pinned macOS launcher."""
import hashlib, struct, sys
from pathlib import Path
exe=Path(sys.argv[1]).read_bytes(); launcher=Path(sys.argv[2]).read_bytes()
magic,size=struct.unpack_from("<II",exe,len(exe)-8)
if magic!=0xFA123456 or size!=len(launcher) or exe[-8-size:-8]!=launcher:
    raise SystemExit("Projector trailer or launcher mismatch")
print("Embedded launcher SHA-256:",hashlib.sha256(launcher).hexdigest())
