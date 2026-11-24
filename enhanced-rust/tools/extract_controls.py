#!/usr/bin/env python3
"""Export original control vector meshes, never execute SWF code.
PKV1: magic, u16 entry count; entry u16 ID, u16 batch count; batch RGBA,
u32 triangle-vertex count, float32 x/y pairs in local stage pixels.
Visible button states are keyed 1000 + character_id*3 + state (up/over/down).
"""
import argparse, hashlib, math, struct, zlib
from pathlib import Path
EXPECTED='1521acdea7d20691da8d917c0e306ce9e39208406377c836b68bc457400c8d8e'
SHAPES=[25,160,161,165,167,170,172,174,179,223]
BUTTONS=[189,231,238]
class Bits:
    def __init__(self,data,p=0):self.data=data;self.at=p*8
    def read(self,n,signed=False):
        if self.at+n>len(self.data)*8:raise ValueError('truncated bits')
        v=0
        for _ in range(n):v=v*2+((self.data[self.at//8]>>(7-self.at%8))&1);self.at+=1
        return v-(1<<n) if signed and n and v&(1<<(n-1)) else v
    def align(self):self.at=(self.at+7)//8*8
    def u8(self):return self.read(8)
    def u16(self):return self.u8()|self.u8()<<8
    def rect(self):
        n=self.read(5);v=[self.read(n,True) for _ in range(4)];self.align();return v
    def matrix(self):
        sx=sy=1;rx=ry=0
        if self.read(1):n=self.read(5);sx=self.read(n,True)/65536;sy=self.read(n,True)/65536
        if self.read(1):n=self.read(5);ry=self.read(n,True)/65536;rx=self.read(n,True)/65536
        n=self.read(5);tx=self.read(n,True);ty=self.read(n,True);self.align();return (sx,sy,rx,ry,tx,ty)
    def cx(self):
        add=self.read(1);mul=self.read(1);n=self.read(4);m=[self.read(n,True) for _ in range(4)] if mul else [256]*4;a=[self.read(n,True) for _ in range(4)] if add else [0]*4;self.align();return m,a

def transform(m,p):sx,sy,rx,ry,tx,ty=m;x,y=p;return sx*x+rx*y+tx,ry*x+sy*y+ty

def tags(data):
    if hashlib.sha256(data).hexdigest()!=EXPECTED:raise ValueError('wrong original SWF')
    b=b'FWS'+data[3:8]+zlib.decompress(data[8:]);assert len(b)==struct.unpack_from('<I',b,4)[0]
    r=Bits(b,8);r.rect();p=r.at//8+4;out={}
    while p+2<=len(b):
        h=struct.unpack_from('<H',b,p)[0];p+=2;c,n=h>>6,h&63
        if n==63:n=struct.unpack_from('<I',b,p)[0];p+=4
        d=b[p:p+n];p+=n
        if c in (2,22,32,11,33,34,75):out[struct.unpack_from('<H',d)[0]]=(c,d)
        if c==0:break
    return out

def edges(r,styles=None):
    # Glyphs have an implicit solid fill 1; shapes provide explicit fill arrays.
    colors=[] if styles else [(0,0,0,255)];es={1:[]} if not styles else {};offset=0
    def read_styles():
        n=r.u8();n=r.u16() if n==255 and styles!=2 else n
        for _ in range(n):
            if r.u8()!=0:raise ValueError('non-solid fill in visible control')
            colors.append(tuple(r.u8() for _ in range(4 if styles==32 else 3)) + (() if styles==32 else (255,)));es[len(colors)]=[]
        if r.u8():raise ValueError('stroke styles not supported in control subset')
        return r.read(4),r.read(4)
    nf,nl=read_styles() if styles else (r.read(4),r.read(4));x=y=0;f0=f1=0
    while True:
        if r.read(1):
            straight=r.read(1);n=r.read(4)+2;old=(x,y);control=None
            if straight:
                if r.read(1):x+=r.read(n,True);y+=r.read(n,True)
                elif r.read(1):y+=r.read(n,True)
                else:x+=r.read(n,True)
            else:
                cx=x+r.read(n,True);cy=y+r.read(n,True);x=cx+r.read(n,True);y=cy+r.read(n,True);control=(cx,cy)
            if f0:es[f0].append((old,control,(x,y)))
            if f1:es[f1].append(((x,y),control,old))
        else:
            flags=r.read(5)
            if not flags:break
            if flags&1:n=r.read(5);x=r.read(n,True);y=r.read(n,True)
            if flags&2:v=r.read(nf);f0=offset+v if v else 0
            if flags&4:v=r.read(nf);f1=offset+v if v else 0
            if flags&8 and r.read(nl):raise ValueError('unsupported stroke')
            if flags&16:
                if not styles:raise ValueError('unexpected glyph style')
                r.align();offset=len(colors);nf,nl=read_styles()
    out=[]
    for i,segs in es.items():
        paths=[]
        while segs:
            a,c,b=segs.pop(0);start=a;path=[(a,c,b)]
            while b!=start:
                j=next((j for j,(aa,_,_) in enumerate(segs) if aa==b),None)
                if j is None:raise ValueError('open control contour')
                a,c,b=segs.pop(j);path.append((a,c,b))
            paths.append(path)
        if paths:out.append((colors[i-1],paths))
    return out

def midpoint(a,b):return (a[0]+b[0])/2,(a[1]+b[1])/2

def curve(a,b,c,out,tol,depth=0):
    ab=midpoint(a,b);bc=midpoint(b,c);m=midpoint(ab,bc);chord=midpoint(a,c)
    if depth>=12 or abs(m[0]-chord[0])+abs(m[1]-chord[1])<=tol:out.append(c);return
    curve(a,ab,m,out,tol,depth+1);curve(m,bc,c,out,tol,depth+1)

def flatten(path,tol):
    out=[path[0][0]]
    for a,b,c in path:
        if b is None:out.append(c)
        else:curve(a,b,c,out,tol)
    return out

def tessellate(paths,tol):
    ps=[flatten(p,tol) for p in paths];ys=sorted({p[1] for path in ps for p in path});es=[(a,b) for path in ps for a,b in zip(path,path[1:]) if a[1]!=b[1]]
    def x(e,y):a,b=e;return a[0]+(b[0]-a[0])*(y-a[1])/(b[1]-a[1])
    out=[]
    for y0,y1 in zip(ys,ys[1:]):
        mid=(y0+y1)/2;active=sorted((e for e in es if min(e[0][1],e[1][1])<mid<max(e[0][1],e[1][1])),key=lambda e:x(e,mid))
        if len(active)%2:raise ValueError('odd intersections')
        for l,r in zip(active[::2],active[1::2]):
            a=(x(l,y0),y0);b=(x(r,y0),y0);c=(x(l,y1),y1);d=(x(r,y1),y1);out.extend((a,c,b,b,c,d))
    return out

def font_glyph(data,index):
    flags=data[2];p=5+data[4];count=struct.unpack_from('<H',data,p)[0];table=p+2;w=4 if flags&8 else 2
    if not 0<=index<count:raise ValueError('glyph index')
    off=lambda i:int.from_bytes(data[table+i*w:table+(i+1)*w],'little')
    raw=data[table+off(index):table+off(index+1)]
    return [p for _,paths in edges(Bits(raw)) for p in paths]

def render_character(ts,id):
    code,data=ts[id];r=Bits(data);r.u16();r.rect()
    if code in (2,22,32):return [(col,tessellate(paths,.2)) for col,paths in edges(r,code)]
    if code not in (11,33):raise ValueError(f'unsupported visible character {id}')
    mat=r.matrix();ng,na=r.u8(),r.u8();out=[];font=None;height=None;x=y=0;col=(0,0,0,255)
    while True:
        flags=r.u8()
        if flags==0:break
        if not flags&128:raise ValueError('bad text record')
        if flags&8:font=r.u16()
        if flags&4:col=tuple(r.u8() for _ in range(4 if code==33 else 3))+(() if code==33 else (255,))
        if flags&1:x=struct.unpack('<h',struct.pack('<H',r.u16()))[0]
        if flags&2:y=struct.unpack('<h',struct.pack('<H',r.u16()))[0]
        if flags&8:height=r.u16()
        n=r.u8();tri=[]
        for _ in range(n):
            i=r.read(ng);advance=r.read(na,True)
            if font is None or height is None or ts[font][0]!=75:raise ValueError('unsupported font')
            paths=font_glyph(ts[font][1],i);scale=height/20480
            tri.extend(transform(mat,(x+px*scale,y+py*scale)) for px,py in tessellate(paths,8))
            x+=advance
        r.align()
        if tri:out.append((col,tri))
    return out

def render_button(ts,id,state):
    code,data=ts[id];assert code==34;r=Bits(data,5);records=[]
    while True:
        flags=r.u8()
        if flags==0:break
        cid,depth=r.u16(),r.u16();m=r.matrix();mul,add=r.cx()
        if flags&48:raise ValueError('unsupported filter/blend')
        if flags&(1<<state):records.append((depth,cid,m,mul,add))
    out=[]
    for _,cid,m,mul,add in sorted(records,key=lambda a:a[0]):
        for col,verts in render_character(ts,cid):
            rgba=tuple(min(255,max(0,round(c*k/256+a))) for c,k,a in zip(col,mul,add));out.append((rgba,[transform(m,p) for p in verts]))
    return out

def pack(data):
    ts=tags(data);entries={id:render_character(ts,id) for id in SHAPES}
    for id in BUTTONS:
        for state in range(3):entries[1000+id*3+state]=render_button(ts,id,state)
    out=bytearray(b'PKV1'+struct.pack('<H',len(entries)))
    for id,batches in sorted(entries.items()):
        out+=struct.pack('<HH',id,len(batches))
        for rgba,verts in batches:
            if not verts or len(verts)%3:raise ValueError('invalid triangle list')
            out+=struct.pack('<4BI',*rgba,len(verts))
            for x,y in verts:out+=struct.pack('<ff',x/20,y/20)
    return bytes(out),entries

def main():
    p=argparse.ArgumentParser();p.add_argument('swf',type=Path);p.add_argument('output',type=Path);a=p.parse_args();data,entries=pack(a.swf.read_bytes());a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_bytes(data);print(f'{len(entries)} vector entries, {sum(len(v)//3 for b in entries.values() for _,v in b)} triangles, {len(data)} bytes; sha256 {hashlib.sha256(data).hexdigest()}')
if __name__=='__main__':main()
