#!/usr/bin/env python3
"""Prepare static artwork-only SWFs for offline PNG rasterization; never execute them."""
import struct,sys,hashlib
from pathlib import Path
from analyze import unpack,u16,u32
DEFINITIONS={2,6,7,8,10,11,13,14,17,20,21,22,23,32,33,34,35,36,37,39,46,48,56,60,62,69,73,74,75,76,77,78,83,84,87,88,90,91}
def tag(code,data=b""):
    n=len(data)
    return (struct.pack("<H",(code<<6)|n) if n<63 else struct.pack("<HI",(code<<6)|63,n))+data
def tags(b,p,end):
    while p+2<=end:
        off=p;h=u16(b,p);p+=2;code,n=h>>6,h&63
        if n==63:n=u32(b,p);p+=4
        e=p+n
        if e>end:raise ValueError("truncated tag")
        yield off,code,b[p:e];p=e
        if code==0:return
def strip_sprite(body):
    # Remove script actions from every nested timeline; drawing tags preserved.
    return body[:4]+b"".join(tag(c,strip_sprite(d) if c==39 else d) for _,c,d in tags(body,4,len(body)) if c not in (12,59))
def write(out,b,header_end,frame_count,body):
    header=b[8:header_end-2]+struct.pack("<H",frame_count)
    out.write_bytes(b"FWS"+b[3:4]+struct.pack("<I",8+len(header)+len(body))+header+body)
def main():
    path,out=Path(sys.argv[1]),Path(sys.argv[2])
    data=path.read_bytes()
    if hashlib.sha256(data).hexdigest()!="1521acdea7d20691da8d917c0e306ce9e39208406377c836b68bc457400c8d8e":raise ValueError("wrong SWF")
    b=unpack(data);header_end=8+(5+4*(b[8]>>3)+7)//8+4
    all_tags=list(tags(b,header_end,len(b)));out.mkdir(parents=True,exist_ok=True)
    defs=[];fg=[];removed=0;bg_count=None
    for off,c,d in all_tags:
        if c==39:
            if u16(d,0)==75:bg_count=u16(d,2)
            d=strip_sprite(d)
        if c in DEFINITIONS:defs.append(tag(c,d))
        if off==2379730:
            if c!=26 or u16(d,1)!=1 or u16(d,3)!=75:raise ValueError("background placement identity")
            removed+=1;continue
        if c not in (12,59):fg.append(tag(c,d))
    if removed!=1 or bg_count!=221:raise ValueError("original background mismatch")
    write(out/"foreground.swf",b,header_end,105,b"".join(fg))
    # Use a synthetic root displaying the original character at its local origin.
    # Matrix 0 is identity, depth 1, PlaceObject2 HasCharacter+HasMatrix.
    for name,cid,count in (("background",75,bg_count),("video_controls",191,1),("gallery_controls",240,1)):
        body=b"".join(defs)+tag(26,struct.pack("<BHHB",6,1,cid,0))+tag(1)*count+tag(0)
        write(out/(name+".swf"),b,header_end,count,body)
    print("Generated 105 foreground frames, 221 background frames and 2 control layers")
if __name__=="__main__":main()
