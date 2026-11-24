#!/usr/bin/env python3
"""Native launcher glyphs from the verified original; no Flash execution."""
import hashlib, struct, subprocess, tempfile, zlib
from pathlib import Path
from PIL import Image
from extract_controls import Bits, render_character, transform
from extract_background import tags
from extract_foreground import tag
from extract_labels import compose
EXPECTED='dc6d49c1629998abd4ddd10538aa081d84ac4779fdd9315ee987f5114e50d1d9'
IDS=(10,15,16)

def build(data, png, pack, jar):
    if hashlib.sha256(data).hexdigest()!=EXPECTED:raise ValueError('wrong launcher SWF')
    raw=b'FWS'+data[3:8]+zlib.decompress(data[8:])
    r=Bits(raw,8);assert r.rect()==[0,10000,0,6000]
    start=r.at//8+4;rows=list(tags(raw,start))
    ts={struct.unpack_from('<H',d)[0]:(c,d) for c,d in rows if c in (2,22,32,11,33,34,75)}
    batches=[];seen=[];body=bytearray()
    root=(1,1,0,0,1248,1360)
    def placement(d):
        r=Bits(d);flags=r.u8();depth=r.u16();cid=r.u16();m=r.matrix()
        return flags,depth,cid,m
    root_rows=[placement(d) for c,d in rows if c==26 and d[0]&2]
    assert [(f,z,m) for f,z,i,m in root_rows if i==17]==[(38,11,root)]
    for c,d in rows:
        if c!=39 or struct.unpack_from('<H',d)[0]!=17:
            body+=tag(c,d);continue
        sprite=bytearray(d[:4])
        assert struct.unpack_from('<H',d,2)[0]==1
        for k,v in tags(d,4):
            if k==26 and v[0]&2:
                f,z,i,m=placement(v)
                if i in IDS:
                    assert f==6
                    seen.append(i)
                    for rgba,points in render_character(ts,i):
                        if not points:continue
                        vertices=[transform(m,p) for p in points]
                        if i==10:
                            # Remaining higher-depth logo #13 is at (1672,0)
                            # with source bounds 4140x1100 twips.
                            assert all(y>=1100 for x,y in vertices)
                        batches.append((rgba,[transform(root,p) for p in vertices]))
                    continue
                assert (i,z,m)==(13,2,(1,1,0,0,1672,0))
            sprite+=tag(k,v)
        body+=tag(c,bytes(sprite))
    assert tuple(seen)==IDS and 0<len(batches)<=32
    stripped=raw[:4]+struct.pack('<I',start+len(body))+raw[8:start]+body
    out=bytearray(b'PKL1'+struct.pack('<HHH',1,1,len(batches)))
    for rgba,points in batches:
        assert 0<len(points)<=100000 and len(points)%3==0
        out+=struct.pack('<4BI',*rgba,len(points))
        for x,y in points:out+=struct.pack('<ff',x/20,y/20)
    assert len(out)<=2*1024*1024
    with tempfile.TemporaryDirectory() as tmp:
        tmp=Path(tmp);swf=tmp/'launcher.swf';swf.write_bytes(stripped)
        target=tmp/'frames';target.mkdir()
        subprocess.run(['java','-Djava.awt.headless=true','-jar',str(jar),'-format','frame:png','-export','frame',str(target),str(swf)],check=True,stdout=subprocess.DEVNULL)
        files=list(target.rglob('*.png'));assert len(files)==1
        with Image.open(files[0]) as im:
            assert im.size==(500,300);im.save(png)
    pack.write_bytes(out)
    print(f'launcher: {len(seen)} original DefineText characters, {sum(len(p)//3 for _,p in batches)} native triangles')
