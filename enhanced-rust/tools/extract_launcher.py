#!/usr/bin/env python3
"""Source-only launcher meshes and clipped original bitmap states. No rasterized launcher UI."""
import hashlib,io,struct,zlib
from PIL import Image
from extract_controls import Bits,render_character,transform,edges
from extract_background import tags
from extract_thumbnails import rectangles
from extract_labels import compose
EXPECTED='dc6d49c1629998abd4ddd10538aa081d84ac4779fdd9315ee987f5114e50d1d9'
IDS=(10,15,16)
def build(data,png,pack,jar):
    assert hashlib.sha256(data).hexdigest()==EXPECTED
    raw=b'FWS'+data[3:8]+zlib.decompress(data[8:]);r=Bits(raw,8);assert r.rect()==[0,10000,0,6000]
    rows=list(tags(raw,r.at//8+4));defs={struct.unpack_from('<H',d)[0]:(c,d) for c,d in rows if c in (2,21,22,32,35,39,11,33,34,75)}
    def placement(d):
        r=Bits(d);f=r.u8();z=r.u16();i=r.u16();m=r.matrix();return f,z,i,m
    root=[placement(d) for c,d in rows if c==26 and d[0]&2]
    assert [(f,z,i) for f,z,i,m in root]==[(38,1,3),(38,3,4),(70,5,5),(38,6,8),(38,8,8),(38,11,17)]
    assert all(m[:4]==(1,1,0,0) for f,z,i,m in root)
    def children(i):
        c,d=defs[i];assert c==39 and struct.unpack_from('<H',d,2)[0]==1
        return [placement(v) for k,v in tags(d,4) if k==26 and v[0]&2]
    def bitmap(shape):
        c,d=defs[shape];assert c in (2,22)
        vals=rectangles(d);assert len(vals)==1
        i,x,y,w,h=vals[0];c,d=defs[i]
        if c==35:
            n=struct.unpack_from('<I',d,2)[0];im=Image.open(io.BytesIO(d[6:6+n])).convert('RGBA')
            alpha=zlib.decompress(d[6+n:]);assert len(alpha)==im.width*im.height
            im.putalpha(Image.frombytes('L',im.size,alpha))
        else:
            assert c==21;im=Image.open(io.BytesIO(d[2:])).convert('RGBA')
        assert (w,h)==(im.width*20,im.height*20)
        return im,(x,y,w,h)
    def clip(rect,mask):
        x,y,w,h=rect;l,t,r,b=mask
        a,c=max(x,l),max(y,t);e,f=min(x+w,r),min(y+h,b)
        assert a<e and c<f
        return ((a/20,c/20,(e-a)/20,(f-c)/20),((a-x)/w,(c-y)/h,(e-x)/w,(f-y)/h))
    def translated(rect,m):
        assert m[:4]==(1,1,0,0)
        x,y,w,h=rect;return (x+m[4],y+m[5],w,h)
    stage=(0,0,10000,6000)
    image_rows=[]
    assert children(3)==[(6,1,2,(1,1,0,0,0,0))]
    im,rect=bitmap(2);image_rows.append((0,im,[clip(translated(rect,root[0][3]),stage)]*3))
    r=Bits(defs[5][1],2);r.rect();mask_paths=edges(r,2)
    assert len(mask_paths)==1
    masks=[]
    for path in mask_paths[0][1]:
        assert len(path)==4 and all(c is None for _,c,_ in path)
        xs=sorted({a[0] for a,_,_ in path});ys=sorted({a[1] for a,_,_ in path});assert len(xs)==len(ys)==2
        masks.append((xs[0],ys[0],xs[1],ys[1]))
    masks.sort();assert masks==[(980,4520,4740,5060),(5180,4520,8940,5060)]
    im,rect=bitmap(7);r=Bits(defs[8][1],5);states=[[] for _ in range(3)]
    while True:
        f=r.u8()
        if not f:break
        assert r.u16()==7 and r.u16()==1
        m=r.matrix();mul,add=r.cx();assert mul==[256]*4 and add==[0]*4
        for i in range(3):
            if f&(1<<i):states[i].append(m)
    assert all(len(v)==1 for v in states)
    for i,mask in enumerate(masks):
        m=root[3+i][3]
        image_rows.append((i+1,im,[clip(translated(rect,compose(m,v[0])),mask) for v in states]))
    batches=[];seen=[];parent=root[-1][3]
    for f,z,i,m in children(17):
        assert f==6
        if i==13:
            assert children(13)==[(6,1,12,(1,1,0,0,0,0))]
            im,rect=bitmap(12);image_rows.append((0,im,[clip(translated(rect,compose(parent,m)),stage)]*3))
        else:
            assert i in IDS;seen.append(i)
            for rgba,points in render_character(defs,i):
                vertices=[transform(m,p) for p in points]
                if i==10:assert all(y>=1100 for x,y in vertices)
                if vertices:batches.append((rgba,[transform(parent,p) for p in vertices]))
    assert tuple(seen)==IDS and len(image_rows)==4
    glyphs=bytearray(b'PKL1'+struct.pack('<HHH',1,1,len(batches)))
    for rgba,points in batches:
        assert 0<len(points)<=100000 and len(points)%3==0
        glyphs+=struct.pack('<4BI',*rgba,len(points))
        for x,y in points:glyphs+=struct.pack('<ff',x/20,y/20)
    out=bytearray(b'PKL2'+struct.pack('<I',len(glyphs))+glyphs+bytes([len(image_rows)]))
    for selector,im,states in image_rows:
        out+=struct.pack('<BHH',selector,im.width,im.height)
        for rect,uv in states:out+=struct.pack('<8f',*rect,*uv)
        out+=im.tobytes()
    assert len(out)<=4*1024*1024
    pack.write_bytes(out)
    # Keep the original launcher PNG only as legacy fallback; PKL2 never draws it.
    print(f'launcher: {len(image_rows)} original bitmap instances, 2 source clipping masks, {sum(len(p)//3 for _,p in batches)} native triangles')
