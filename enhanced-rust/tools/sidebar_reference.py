#!/usr/bin/env python3
"""Isolate original sidebar #50 with unchanged masks/buttons, with and without blur.
The emitted fixtures contain no ActionScript, external references, or media streams.
They are reference-only; no Flash runtime is used by the reconstructed application.
"""
import struct,zlib,hashlib,sys
from pathlib import Path
from extract_controls import Bits,EXPECTED
from extract_foreground import tag,bits
from extract_background import tags

def build(raw,dest):
    assert hashlib.sha256(raw).hexdigest()==EXPECTED
    b=b'FWS'+raw[3:8]+zlib.decompress(raw[8:])
    r=Bits(b,8);assert r.rect()==[0,25600,0,14400]
    header=r.at//8+4
    ids={25,26,27,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50}
    defs=[];found=set()
    for c,d in tags(b,header):
        if c==8:defs.append(tag(c,d));continue
        if c not in (2,6,11,21,22,32,33,34,35,39,75) or len(d)<2:continue
        cid=struct.unpack_from('<H',d)[0]
        if cid not in ids:continue
        assert cid not in found;found.add(cid)
        if c==34:
            # All four original buttons have no action records.
            assert cid in (29,36,39,42) and struct.unpack_from('<H',d,3)[0]==0
            p=Bits(d,5)
            while (f:=p.u8()):
                assert not f&48 and p.u16() in ids
                p.u16();p.matrix();p.cx()
            assert p.at//8==len(d)
        if c==39:
            assert cid in (45,50) and struct.unpack_from('<H',d,2)[0]==1
            children=[]
            for k,v in tags(d,4):
                if k in (12,45,59):continue
                assert k in (0,1,26)
                if k==26:
                    p=Bits(v);flags=p.u8();assert flags in (6,38,70);p.u16();assert p.u16() in ids
                    p.matrix()
                children.append(tag(k,v))
            d=d[:4]+b''.join(children)
        defs.append(tag(c,d))
    assert found==ids,(found,ids-found)
    # Exact placement of settled source sidebar at x=4460, y=460 twips.
    matrix=bits((0,1),(0,1),(17,5),(4460,17),(460,17))
    common=tag(9,b'\0\0\0')+b''.join(defs)
    plain=tag(26,struct.pack('<BHH',6,1,50)+matrix)
    blur=tag(70,struct.pack('<BBHH',6,1,1,50)+matrix+b'\1\1'+struct.pack('<II',0,15*65536)+b'\x08')
    dest.mkdir(parents=True,exist_ok=True)
    for name,place in [('sidebar-plain',plain),('sidebar-blur',blur)]:
        body=b[8:header-2]+struct.pack('<H',1)+common+place+tag(1)+tag(0)
        (dest/(name+'.swf')).write_bytes(b'FWS'+b[3:4]+struct.pack('<I',8+len(body))+body)
    print('Built two isolated, action-free source sidebar fixtures')
if __name__=='__main__':build(Path(sys.argv[1]).read_bytes(),Path(sys.argv[2]))
