"""Compact original DefineText/DefineFont3 glyph dictionary for menu panels.
No raster text or repeated tessellated glyphs in the runtime asset.
"""
import struct
from extract_controls import Bits
from extract_background import tags
LABELS={157:(143,144,145,146,147,148,149,156),220:(213,214,215),136:(119,123,124,125,129,130,131,135)}

def glyph_bytes(d,i):
    flags=d[2];p=5+d[4];count=struct.unpack_from('<H',d,p)[0];table=p+2;w=4 if flags&8 else 2
    assert 0<=i<count
    off=lambda j:int.from_bytes(d[table+j*w:table+(j+1)*w],'little')
    return d[table+off(i):table+off(i+1)]

def compose(a,b):
    sx,sy,rx,ry,tx,ty=a;u,v,p,q,x,y=b
    return sx*u+rx*q,ry*p+sy*v,sx*p+rx*v,ry*u+sy*q,sx*x+rx*y+tx,ry*x+sy*y+ty

def label_records(sprite):
    sid=struct.unpack_from('<H',sprite)[0];out=[]
    for c,d in tags(sprite,4):
        if c!=26 or not d[0]&2:continue
        r=Bits(d);flags=r.u8();r.u16();cid=r.u16()
        if cid not in LABELS[sid]:continue
        assert flags==6;m=r.matrix();assert r.at//8==len(d)
        out.append((cid,m))
    assert tuple(cid for cid,_ in out)==LABELS[sid]
    return out

def strip(sprite):
    sid=struct.unpack_from('<H',sprite)[0]
    if sid not in LABELS:return sprite
    label_records(sprite)
    from extract_foreground import tag
    return sprite[:4]+b''.join(tag(c,d) for c,d in tags(sprite,4) if not(c==26 and d[0]&2 and struct.unpack_from('<H',d,3)[0] in LABELS[sid]))

def pack_labels(ts,sprites):
    glyphs=[];lookup={};groups=[]
    for sid in LABELS:
        rows=[]
        for cid,placement in label_records(sprites[sid]):
            code,data=ts[cid];assert code in (11,33);r=Bits(data,2);r.rect();m=compose((1,1,0,0,10537,860),compose(placement,r.matrix()));ng,na=r.u8(),r.u8();font=height=None;x=y=0;rgba=(0,0,0,255)
            while True:
                flags=r.u8()
                if flags==0:break
                assert flags&128 and not flags&0x70
                if flags&8:font=r.u16()
                if flags&4:rgba=tuple(r.u8() for _ in range(4 if code==33 else 3))+(() if code==33 else (255,))
                if flags&1:x=struct.unpack('<h',struct.pack('<H',r.u16()))[0]
                if flags&2:y=struct.unpack('<h',struct.pack('<H',r.u16()))[0]
                if flags&8:height=r.u16()
                count=r.u8();assert ts[font][0]==75 and height
                for _ in range(count):
                    i=r.read(ng);advance=r.read(na,True);raw=glyph_bytes(ts[font][1],i)
                    if raw not in lookup:lookup[raw]=len(glyphs);glyphs.append(raw)
                    scale=height/20480;transform=compose(m,(scale,scale,0,0,x,y));rows.append((lookup[raw],rgba,tuple(v/20 for v in transform)));x+=advance
                r.align()
        groups.append(rows)
    out=bytearray(struct.pack('<H',len(glyphs)))
    for raw in glyphs:out+=struct.pack('<H',len(raw))+raw
    for rows in groups:
        out+=struct.pack('<H',len(rows))
        for i,rgba,m in rows:out+=struct.pack('<H4B6f',i,*rgba,*m)
    print(f'{len(glyphs)} original glyphs, {sum(map(len,groups))} glyph placements, {len(out)} label bytes')
    return bytes(out)
