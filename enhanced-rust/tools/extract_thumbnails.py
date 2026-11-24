#!/usr/bin/env python3
"""Extract the original rectangular JPEG fills from three menu sprites."""
import hashlib, io, struct, zlib
from PIL import Image
from extract_controls import Bits, EXPECTED
from extract_background import tags
SHAPES={136:134,157:142,220:212}

def rectangles(d):
    r=Bits(d,2);r.rect();styles=[];offset=0;edges={};x=y=0;f0=f1=0
    def read_styles():
        n=r.u8();assert n==2
        for _ in range(n):
            assert r.u8()==0x43
            cid=r.u16();m=r.matrix();assert m[:4]==(20,20,0,0)
            styles.append((cid,m));edges[len(styles)]=[]
        assert r.u8()==0
        return r.read(4),r.read(4)
    nf,nl=read_styles()
    while True:
        if r.read(1):
            assert r.read(1)==1; n=r.read(4)+2;old=(x,y)
            if r.read(1):x+=r.read(n,True);y+=r.read(n,True)
            elif r.read(1):y+=r.read(n,True)
            else:x+=r.read(n,True)
            assert x==old[0] or y==old[1]
            if f0:edges[f0].append((old,(x,y)))
            if f1:edges[f1].append(((x,y),old))
        else:
            flags=r.read(5)
            if not flags:break
            if flags&1:n=r.read(5);x=r.read(n,True);y=r.read(n,True)
            if flags&2:v=r.read(nf);f0=offset+v if v else 0
            if flags&4:v=r.read(nf);f1=offset+v if v else 0
            if flags&8:assert r.read(nl)==0
            if flags&16:r.align();offset=len(styles);nf,nl=read_styles()
    out=[]
    for i,(cid,m) in enumerate(styles,1):
        es=edges[i]
        if cid==65535:assert not es;continue
        assert len(es)==4 and {a for a,b in es}=={b for a,b in es}
        xs=sorted({a[0] for a,b in es});ys=sorted({a[1] for a,b in es})
        assert len(xs)==len(ys)==2 and {a for a,b in es}=={(x,y) for x in xs for y in ys}
        assert m[4:]==(xs[0],ys[0])
        out.append((cid,xs[0],ys[0],xs[1]-xs[0],ys[1]-ys[0]))
    return out

def extract(data):
    assert hashlib.sha256(data).hexdigest()==EXPECTED
    b=b'FWS'+data[3:8]+zlib.decompress(data[8:]);r=Bits(b,8);r.rect()
    ts={struct.unpack_from('<H',d)[0]:(c,d) for c,d in tags(b,r.at//8+4) if c in (2,21,22,39)}
    out={}
    for sid,shape in SHAPES.items():
        code,sprite=ts[sid];assert code==39
        places=[]
        for c,d in tags(sprite,4):
            if c==26 and d[0]&2 and struct.unpack_from('<H',d,3)[0]==shape:
                r=Bits(d);assert r.u8()==6;r.u16();assert r.u16()==shape;assert r.matrix()==(1,1,0,0,0,0);places.append(1)
        assert len(places)==1
        code,d=ts[shape];assert code in (2,22);rows=[]
        for cid,x,y,w,h in rectangles(d):
            code,jpeg=ts[cid];assert code==21;raw=jpeg[2:];assert raw[:2]==b'\xff\xd8'
            with Image.open(io.BytesIO(raw)) as im:
                assert im.format=='JPEG';iw,ih=im.size
            assert (w,h)==(iw*20,ih*20),(cid,w,h,iw,ih)
            rows.append((cid,x,y,w,h,iw,ih,raw))
        out[sid]=rows
    assert [len(out[s]) for s in SHAPES]==[1,5,20]
    return out

def strip(sprite):
    sid=struct.unpack_from('<H',sprite)[0]
    if sid not in SHAPES:return sprite
    from extract_foreground import tag
    return sprite[:4]+b''.join(tag(c,d) for c,d in tags(sprite,4) if not(c==26 and d[0]&2 and struct.unpack_from('<H',d,3)[0]==SHAPES[sid]))
