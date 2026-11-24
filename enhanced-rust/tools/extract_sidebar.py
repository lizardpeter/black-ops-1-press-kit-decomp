"""Direct source bitmap masks and text for the settled original sidebar #50."""
import hashlib,io,struct,zlib
from PIL import Image
from extract_controls import Bits,EXPECTED,tags as dictionary,render_character,transform
from extract_background import tags
from extract_thumbnails import rectangles

def extract(data):
    if hashlib.sha256(data).hexdigest()!=EXPECTED:raise ValueError('wrong original SWF')
    raw=b'FWS'+data[3:8]+zlib.decompress(data[8:]);r=Bits(raw,8);r.rect()
    rows=list(tags(raw,r.at//8+4))
    defs={struct.unpack_from('<H',d)[0]:(c,d) for c,d in rows if c in (2,6,21,22,32,34,35,39)}
    ts=dictionary(data);out=[]
    def image(cid):
        code,d=defs[cid]
        if code==35:
            n=struct.unpack_from('<I',d,2)[0]
            im=Image.open(io.BytesIO(d[6:6+n])).convert('RGBA')
            a=zlib.decompress(d[6+n:]);assert len(a)==im.width*im.height
            im.putalpha(Image.frombytes('L',im.size,a));return im
        assert code in (6,21)
        return Image.open(io.BytesIO(d[2:])).convert('RGBA')
    def bitmap(shape):
        code,d=defs[shape];assert code in (2,22)
        v=rectangles(d);assert len(v)==1
        cid,x,y,w,h=v[0];im=image(cid)
        assert (w,h)==(20*im.width,20*im.height)
        return cid,(x,y,w,h),im
    def button(id):
        code,d=defs[id];assert code==34
        r=Bits(d,5);states=[[] for _ in range(3)]
        while True:
            flags=r.u8()
            if not flags:break
            cid,depth=r.u16(),r.u16();m=r.matrix();mul,add=r.cx()
            assert flags<16 and mul==[256]*4 and add==[0]*4
            for i in range(3):
                if flags&(1<<i):states[i].append((cid,m))
        assert all(len(s)==1 for s in states)
        return [s[0] for s in states]
    def bounds(shape,m):
        code,d=defs[shape];assert code==2;r=Bits(d,2);x0,x1,y0,y1=r.rect()
        assert m[2:4]==(0,0) and m[0]>0 and m[1]>0
        a=transform(m,(x0,y0));b=transform(m,(x1,y1));return a[0],a[1],b[0],b[1]
    def place(d):
        r=Bits(d);f=r.u8();depth=r.u16();cid=r.u16();m=r.matrix();return f,depth,cid,m
    def clip(rect,mask):
        x,y,w,h=rect;l,t,r,b=mask
        x0,y0=max(l,x),max(t,y);x1,y1=min(r,x+w),min(b,y+h)
        assert x1>x0 and y1>y0
        return ((x0/20,y0/20,(x1-x0)/20,(y1-y0)/20),((x0-x)/w,(y0-y)/h,(x1-x)/w,(y1-y)/h))
    sprite=defs[50][1];end=0;mask=None;last=0
    for c,d in tags(sprite,4):
        if c in (0,1,12,45,59):continue
        assert c==26
        flags,depth,cid,m=place(d);assert depth>last;last=depth
        if cid==25:
            assert flags==70;end=struct.unpack_from('<H',d,len(d)-2)[0]
            assert end>depth;mask=bounds(cid,m);continue
        if cid in (29,36,39,42):
            assert flags==38 and depth<=end and mask is not None
            assert m[:4]==(1,1,0,0)
            states=[];img=None;ident=None
            for shape,bm in button(cid):
                assert bm[:4]==(1,1,0,0)
                bid,(x,y,w,h),im=bitmap(shape)
                if ident is None:ident=bid;img=im
                assert ident==bid
                states.append(clip((x+m[4]+bm[4],y+m[5]+bm[5],w,h),mask))
            key={29:240,36:241,39:242,42:243}[cid]
            out.append(('texture',key,(img,states)))
        elif cid==45:
            assert flags==6 and depth>end and m[:4]==(1,1,0,0)
            code,s=defs[cid];assert code==39
            child=[place(v) for k,v in tags(s,4) if k==26]
            assert child==[(6,1,44,(1,1,0,0,0,0))]
            _,(x,y,w,h),im=bitmap(44)
            state=(( (x+m[4])/20,(y+m[5])/20,w/20,h/20),(0.,0.,1.,1.))
            out.append(('texture',244,(im,[state]*3)))
        elif cid in (31,33,47,49):
            assert flags==6 and depth>end
            batches=[(rgba,[transform(m,p) for p in points]) for rgba,points in render_character(ts,cid) if points]
            assert batches
            out.append(('mesh',64000+cid,batches))
        else:raise ValueError(f'unknown sidebar child {cid}')
    assert [key for _,key,_ in out]==[240,64031,64033,241,242,243,244,64047,64049]
    return out
