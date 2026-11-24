#!/usr/bin/env python3
"""Extract original menu button states and placement records; no runtime SWF."""
import struct, zlib
from extract_controls import Bits, EXPECTED, tags as dictionary, render_button
from extract_background import tags
import hashlib
BUTTONS={157:(155,'trl_',(1,2,4,5,6)),220:(219,'scrn_',tuple(range(1,21)))}

def button_placements(sprite):
    sid=struct.unpack_from('<H',sprite)[0]
    if sid not in BUTTONS:return []
    char,prefix,ids=BUTTONS[sid];rows=[]
    for c,d in tags(sprite,4):
        if c!=26 or len(d)<5 or not d[0]&2 or struct.unpack_from('<H',d,3)[0]!=char:continue
        r=Bits(d);assert r.u8()==38;r.u16();assert r.u16()==char
        m=r.matrix();assert m[:4]==(1,1,0,0)
        name=d[r.at//8:].rstrip(b'\0').decode('ascii');assert name.startswith(prefix)
        rows.append((int(name[len(prefix):]),m[4],m[5]))
    assert tuple(row[0] for row in rows)==ids
    return rows

def strip_buttons(sprite):
    sid=struct.unpack_from('<H',sprite)[0]
    if sid not in BUTTONS:return None
    button_placements(sprite);char=BUTTONS[sid][0]
    from extract_foreground import tag
    return sprite[:4]+b''.join(tag(c,d) for c,d in tags(sprite,4) if not(c==26 and len(d)>=5 and d[0]&2 and struct.unpack_from('<H',d,3)[0]==char) and c not in (12,59))

def pack(data,frames):
    assert hashlib.sha256(data).hexdigest()==EXPECTED
    raw=b'FWS'+data[3:8]+zlib.decompress(data[8:]);r=Bits(raw,8);r.rect()
    sprites={struct.unpack_from('<H',d)[0]:d for c,d in tags(raw,r.at//8+4) if c==39}
    ts=dictionary(data);out=bytearray(b'PKM1');groups=[]
    for sid in BUTTONS:
        placements=button_placements(sprites[sid]);groups.append(placements)
        out.append(len(placements))
        for row in placements:out+=struct.pack('<Bii',*row)
    for frame in frames:
        matches=[(cid,m,a,flt) for cid,m,a,flt,_ in frame if cid in BUTTONS]
        assert len(matches)<=1
        if not matches:out.append(0);continue
        cid,m,a,flt=matches[0];assert [c for c,_,a,_,_ in frame if a][:4]==[96,80,99,cid];assert m==(1,1,0,0,10537,860) and a==256 and not flt
        out.append(1 if cid==157 else 2)
    for sid in BUTTONS:
        for state in range(3):
            batches=render_button(ts,BUTTONS[sid][0],state);out.append(len(batches))
            for rgba,verts in batches:
                assert len(verts)%3==0 and verts
                out+=struct.pack('<4BH',*rgba,len(verts))
                for x,y in verts:out+=struct.pack('<ff',x/20,y/20)
    return bytes(out)
