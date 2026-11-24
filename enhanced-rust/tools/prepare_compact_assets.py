#!/usr/bin/env python3
"""Build an independent Rust runtime asset directory, omitting all baked background/full-frame references."""
import argparse, hashlib, shutil
from pathlib import Path
from extract_background import extract
from extract_controls import pack

def prepare(swf:Path,reference:Path,dest:Path):
    reference=reference.resolve();dest=dest.resolve()
    if dest.exists() or reference in dest.parents or dest in reference.parents:raise ValueError('destination must be a new separate directory')
    raw=swf.read_bytes();timeline,jpegs,_=extract(raw);controls,_=pack(raw)
    paths=[Path('art/launcher/1.png'),*(Path(f'art/foreground/{i}.png') for i in range(1,106)),*(Path(f'art/sounds/{i}.mp3') for i in (1,2,28,52,154))]
    if not all((reference/p).is_file() for p in paths) or not (reference/'payload').is_dir():raise ValueError('missing original foreground, launcher, audio or payload')
    dest.mkdir(parents=True)
    for p in paths:
        (dest/p).parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(reference/p,dest/p)
    shutil.copytree(reference/'payload',dest/'payload')
    # Original external payload BG duplicates the embedded JPEGs; retain the original payload intact.
    bg=dest/'art/background-source';bg.mkdir();(bg/'timeline.pbg').write_bytes(timeline)
    for i,data in enumerate(jpegs):(bg/f'{i}.jpg').write_bytes(data)
    vector=dest/'art/vector';vector.mkdir();(vector/'controls.pkv').write_bytes(controls)
    assert not (dest/'art/background').exists() and not (dest/'art/presskit').exists()
    print(f'compact background: {sum(map(len,jpegs))+len(timeline)} bytes; runtime art: {sum(p.stat().st_size for p in (dest/"art").rglob("*") if p.is_file())} bytes')

def main():
    p=argparse.ArgumentParser();p.add_argument('swf',type=Path);p.add_argument('reference',type=Path);p.add_argument('destination',type=Path);a=p.parse_args();prepare(a.swf,a.reference,a.destination)
if __name__=='__main__':main()
