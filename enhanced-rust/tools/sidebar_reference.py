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

def flatten_reference_masks(b,rows):
    """Reference-only replacement of each static up-state masked bitmap by
    a rectangular DefineShape2 using the same original bitmap and transform.
    Geometry is derived directly from source SWF records, not the native pack.
    Preserve every unmasked depth and the original enclosing blur.
    """
    from extract_thumbnails import rectangles
    from extract_controls import edges
    defs={struct.unpack_from('<H',d)[0]:(c,d) for c,d in rows if c in (2,6,11,21,22,32,33,34,35,39,75)}
    assert all(i not in defs for i in range(65010,65014))
    def rectangle_shape(cid,bitmap,origin,box):
        x0,y0,x1,y1=box;tx,ty=origin
        assert x0<x1 and y0<y1
        # DefineShape2: source 0x43 clipped/non-smoothed bitmap fill.
        bounds=bits((17,5),(x0,17),(x1,17),(y0,17),(y1,17))
        matrix=bits((1,1),(23,5),(20*65536,23),(20*65536,23),(0,1),(17,5),(tx,17),(ty,17))
        fields=[(1,4),(0,4),(0,1),(5,5),(17,5),(x0,17),(y0,17),(1,1)]
        # Clockwise edges, interior on fillStyle1 side.
        for delta,vertical in [(x1-x0,0),(y1-y0,1),(x0-x1,0),(y0-y1,1)]:
            assert -65536<=delta<65536
            fields.extend([(1,1),(1,1),(15,4),(0,1),(vertical,1),(delta,17)])
        fields.extend([(0,1),(0,5)])
        body=struct.pack('<H',cid)+bounds+b'\x01\x43'+struct.pack('<H',bitmap)+matrix+b'\x00'+bits(*fields)
        assert rectangles(body)==[(bitmap,x0,y0,x1-x0,y1-y0)] if origin==(x0,y0) else True
        return tag(22,body)
    sprite=defs[50][1];new=[];out=[];end=0;mask=None;seen=[]
    for c,d in tags(sprite,4):
        if c!=26:
            if c not in (12,45,59):out.append(tag(c,d))
            continue
        r=Bits(d);flags=r.u8();depth=r.u16();cid=r.u16();m=r.matrix()
        sx,sy,rx,ry,tx,ty=m
        if cid==25:
            assert flags==70 and rx==ry==0 and sx>0 and sy>0
            q=Bits(defs[25][1],2);q.rect()
            fills=edges(q,2);assert len(fills)==1 and len(fills[0][1])==1
            contour=fills[0][1][0]
            assert len(contour)==4 and all(k is None for _,k,_ in contour)
            xs=sorted({p[0] for p,_,_ in contour});ys=sorted({p[1] for p,_,_ in contour})
            assert len(xs)==len(ys)==2
            mask=(tx+xs[0]*sx,ty+ys[0]*sy,tx+xs[1]*sx,ty+ys[1]*sy)
            end=r.u16();assert depth<end
            continue
        if depth<=end:
            assert flags==38 and cid in (29,36,39,42) and m[:4]==(1,1,0,0)
            q=Bits(defs[cid][1],5);up=[]
            while (f:=q.u8()):
                shape,z=q.u16(),q.u16();bm=q.matrix();mul,offset=q.cx()
                assert not f&48 and mul==[256]*4 and offset==[0]*4
                if f&1:up.append((shape,bm))
            assert len(up)==1
            shape,bm=up[0];assert bm[:4]==(1,1,0,0)
            rect=rectangles(defs[shape][1]);assert len(rect)==1
            bid,x,y,w,h=rect[0];x+=tx+bm[4];y+=ty+bm[5]
            l,t,rr,bb=mask
            box=tuple(round(a) for a in (max(x,l),max(y,t),min(x+w,rr),min(y+h,bb)))
            new_id=65010+len(seen);seen.append(cid)
            new.append(rectangle_shape(new_id,bid,(x,y),box))
            out.append(tag(26,struct.pack('<BHHB',6,depth,new_id,0)))
        else:out.append(tag(c,d))
    assert seen==[29,36,39,42]
    return b''.join(new),sprite[:4]+b''.join(out)

def build(raw,dest):
    assert hashlib.sha256(raw).hexdigest()==EXPECTED
    b=b'FWS'+raw[3:8]+zlib.decompress(raw[8:])
    r=Bits(b,8);assert r.rect()==[0,25600,0,14400]
    header=r.at//8+4
    ids={25,26,27,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50}
    rows=list(tags(b,header));flat_defs,flat_sprite=flatten_reference_masks(b,rows)
    defs=[];flat=[];found=set()
    for c,d in rows:
        if c==8:defs.append(tag(c,d));flat.append(tag(c,d));continue
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
        if cid==50:flat.append(flat_defs);flat.append(tag(c,flat_sprite))
        else:flat.append(tag(c,d))
    assert found==ids,(found,ids-found)
    # Exact placement of settled source sidebar at x=4460, y=460 twips.
    matrix=bits((0,1),(0,1),(17,5),(4460,17),(460,17))
    common=tag(9,b'\0\0\0')+b''.join(defs)
    plain=tag(26,struct.pack('<BHH',6,1,50)+matrix)
    blur=tag(70,struct.pack('<BBHH',6,1,1,50)+matrix+b'\1\1'+struct.pack('<II',0,15*65536)+b'\x08')
    dest.mkdir(parents=True,exist_ok=True)
    for name,place,definitions in [('sidebar-plain',plain,common),('sidebar-blur',blur,common),('sidebar-flat-plain',plain,tag(9,b'\0\0\0')+b''.join(flat)),('sidebar-flat-blur',blur,tag(9,b'\0\0\0')+b''.join(flat))]:
        body=b[8:header-2]+struct.pack('<H',1)+definitions+place+tag(1)+tag(0)
        (dest/(name+'.swf')).write_bytes(b'FWS'+b[3:4]+struct.pack('<I',8+len(body))+body)
    print('Built two isolated, action-free source sidebar fixtures')
def rewrite_reference(original,src,dest):
    """Correct only the action-free foreground oracle. Runtime assets are untouched."""
    assert hashlib.sha256(original).hexdigest()==EXPECTED
    source=b'FWS'+original[3:8]+zlib.decompress(original[8:])
    r=Bits(source,8);r.rect();header=r.at//8+4
    clipped,sidebar=flatten_reference_masks(source,list(tags(source,header)))
    b=src.read_bytes();assert b[:3]==b'FWS'
    r=Bits(b,8);assert r.rect()==[0,25600,0,14400];header=r.at//8+4
    assert struct.unpack_from('<H',b,header-2)[0]==105
    out=[];replaced=0;frames=0
    for c,d in tags(b,header):
        assert c not in (12,59),'reference must be action-free'
        if c==39 and struct.unpack_from('<H',d)[0]==50:
            out.append(clipped);out.append(tag(39,sidebar));replaced+=1
        else:out.append(tag(c,d))
        if c==1:frames+=1
    assert replaced==1 and frames==105
    body=b[8:header]+b''.join(out)
    dest.write_bytes(b'FWS'+b[3:4]+struct.pack('<I',8+len(body))+body)
    print('Corrected reference-only mask geometry; preserved all 105 frames and source filter records')

if __name__=='__main__':
    if len(sys.argv)==4:rewrite_reference(Path(sys.argv[1]).read_bytes(),Path(sys.argv[2]),Path(sys.argv[3]))
    else:build(Path(sys.argv[1]).read_bytes(),Path(sys.argv[2]))

