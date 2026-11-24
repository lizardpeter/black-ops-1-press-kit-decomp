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
    def visit(cid,m,out):
        code,d=defs[cid]
        if code==39:
            assert struct.unpack_from('<H',d,2)[0]==1
            depth=0
            for c,v in tags(d,4):
                if c in (0,1,12,45,59):continue
                assert c==26
                r=Bits(v);assert r.u8()==6;dep=r.u16();i=r.u16();local=r.matrix()
                assert dep>depth and r.at//8==len(v);depth=dep
                visit(i,compose(m,local),out)
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
                    im=image(i);assert (xs[1]-xs[0],ys[1]-ys[0])==(im.width*20,im.height*20)
                    assert m[:4]==(1,1,0,0)
                    x,y=transform(m,(xs[0],ys[0]))
                    out.append(('bitmap',i,x,y,im.width*20,im.height*20))
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
    return images,meshes,roots
