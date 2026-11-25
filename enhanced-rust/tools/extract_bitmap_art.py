"""Original static sprite geometry: bitmap fills remain source images; solid fills are triangles."""
import hashlib,io,struct,zlib
from PIL import Image
from extract_controls import Bits,EXPECTED,tags as dictionary,edges,tessellate,render_character,transform
from extract_background import tags
from extract_labels import compose
ROOTS=(53,62,80,96,99,111)
def extract(data):
    assert hashlib.sha256(data).hexdigest()==EXPECTED
    b=b'FWS'+data[3:8]+zlib.decompress(data[8:]);r=Bits(b,8);r.rect()
    defs={struct.unpack_from('<H',d)[0]:(c,d) for c,d in tags(b,r.at//8+4) if c in (2,6,21,22,32,35,39,11,33,75)}
    ts=dictionary(data);images={};meshes={};roots={}
    def image(i):
        if i not in images:
            code,d=defs[i]
            if code==35:
                n=struct.unpack_from('<I',d,2)[0];im=Image.open(io.BytesIO(d[6:6+n])).convert('RGBA')
                a=zlib.decompress(d[6+n:]);assert len(a)==im.width*im.height
                im.putalpha(Image.frombytes('L',im.size,a))
            else:
                assert code in (6,21);im=Image.open(io.BytesIO(d[2:])).convert('RGBA')
            images[i]=im
        return images[i]
    def visit(cid,m,out,scaled=False):
        code,d=defs[cid]
        if code==39:
            assert struct.unpack_from('<H',d,2)[0]==1
            depth=0
            for c,v in tags(d,4):
                if c in (0,1,12,45,59):continue
                assert c==26
                r=Bits(v);assert r.u8()==6;dep=r.u16();i=r.u16();local=r.matrix()
                assert dep>depth and r.at//8==len(v);depth=dep
                visit(i,compose(m,local),out,scaled)
            return
        if code in (11,33):
            batches=render_character(ts,cid)
        else:
            assert code in (2,22,32)
            r=Bits(d,2);r.rect();batches=[]
            for fill,paths in edges(r,code,bitmap_fills=True):
                if fill[0]=='bitmap':
                    _,i,mat=fill;assert mat[:4]==(20,20,0,0) and len(paths)==1
                    path=paths[0];assert len(path)==4 and all(c is None for _,c,_ in path)
                    xs=sorted({a[0] for a,_,_ in path});ys=sorted({a[1] for a,_,_ in path})
                    assert len(xs)==len(ys)==2 and mat[4:]==(xs[0],ys[0])
                    im=image(i);w,h=xs[1]-xs[0],ys[1]-ys[0]
                    # Source #104 clips one column of JPEG #103 (572x481).
                    assert (w,h)==(im.width*20,im.height*20) or (cid,i,w,h,im.size)==(104,103,11420,9620,(572,481)),(cid,i,w,h,im.size)
                    assert m[2:4]==(0,0) and m[0]>0 and m[1]>0
                    if not scaled:assert m[:4]==(1,1,0,0)
                    x,y=transform(m,(xs[0],ys[0]))
                    out.append(('bitmap',i,x,y,w*m[0],h*m[1]))
                else:
                    batches.append((fill,tessellate(paths,.2)))
        if batches:
            # Only one mixed fill shape (#85); its solid fill follows its bitmap.
            key=61000+len(meshes);meshes[key]=[(rgba,[transform(m,p) for p in points]) for rgba,points in batches if points]
            assert meshes[key]
            out.append(('mesh',key,0,0,20,20))
    for cid in ROOTS:
        out=[];visit(cid,(1,1,0,0,0,0),out);assert out;roots[cid]=out
    assert 84 in images and any(kind=='mesh' for kind,*_ in roots[96])
    from extract_labels import LABELS
    from extract_thumbnails import SHAPES
    menus={}
    for sid in (136,157,220):
        c,d=defs[sid];out=[]
        for c,v in tags(d,4):
            if c!=26 or not v[0]&2:continue
            r=Bits(v);flags=r.u8();depth=r.u16();i=r.u16()
            if i in (122,128):
                assert flags==6;m=r.matrix();assert r.at//8==len(v)
                visit(i,m,out,True)
            else:assert i in LABELS[sid] or i in (118,132,155,219,SHAPES[sid])
        assert len(out)==(9 if sid==136 else 1)
        assert all(row[0]=='bitmap' for row in out)
        menus[sid]=out
    return images,meshes,roots,menus
