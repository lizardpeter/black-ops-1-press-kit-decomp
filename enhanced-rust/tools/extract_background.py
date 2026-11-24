#!/usr/bin/env python3
"""Extract four untouched original JPEGs and the exact sprite-75 opacity timeline.
PBG1: magic, u16 frame count, then per frame u8 count and (u8 image index,u16 alpha/256) in depth order.
Reject unexpected transformations, masks, blends, or tags rather than approximating them.
"""
import argparse, hashlib, struct, zlib
from pathlib import Path
from extract_controls import Bits, EXPECTED
IMAGES=(63,66,69,72)
SPRITES=(65,68,71,74)
IDENTITY=(1,1,0,0,0,0)
def tags(b,p=0):
    while p+2<=len(b):
        h=struct.unpack_from('<H',b,p)[0];p+=2;c,n=h>>6,h&63
        if n==63:n=struct.unpack_from('<I',b,p)[0];p+=4
        if p+n>len(b):raise ValueError('truncated tag')
        yield c,b[p:p+n];p+=n
        if c==0:return
    raise ValueError('missing end tag')
def sprite_frames(data):
    count=struct.unpack_from('<H',data,2)[0];layers={};frames=[]
    for c,d in tags(data,4):
        if c==26:
            r=Bits(d);flags=r.u8();depth=r.u16()
            if flags&~31:raise ValueError(f'unsupported placement flags {flags}')
            old=layers.get(depth) if flags&1 else None
            cid,mat,alpha=old or (None,IDENTITY,256)
            if flags&2:cid=r.u16()
            if flags&4:mat=r.matrix()
            if flags&8:
                mul,add=r.cx()
                if mul[:3]!=[256]*3 or add!=[0]*4:raise ValueError('unsupported color transform')
                alpha=mul[3]
            if flags&16:r.u16()
            if mat!=IDENTITY or cid is None or not 0<=alpha<=256:raise ValueError('unexpected background transform')
            layers[depth]=(cid,mat,alpha)
        elif c==28:layers.pop(struct.unpack('<H',d)[0])
        elif c==1:frames.append([(cid,a) for cid,_,a in (layers[k] for k in sorted(layers))])
        elif c not in (0,12,45):raise ValueError(f'unexpected sprite tag {c}')
    if len(frames)!=count:raise ValueError('frame count')
    return frames
def extract(data):
    if hashlib.sha256(data).hexdigest()!=EXPECTED:raise ValueError('wrong original SWF')
    b=b'FWS'+data[3:8]+zlib.decompress(data[8:]);r=Bits(b,8);assert r.rect()==[0,25600,0,14400];ts={}
    for c,d in tags(b,r.at//8+4):
        if c in (2,21,39):ts[struct.unpack_from('<H',d)[0]]=(c,d)
    jpegs=[]
    for image,sprite in zip(IMAGES,SPRITES):
        shape=sprite-1;c,d=ts[image];assert c==21 and d[2:4]==b'\xff\xd8';jpegs.append(d[2:])
        c,d=ts[shape];assert c==2;r=Bits(d,2);assert r.rect()==[0,25600,0,14400];assert r.u8()==1 and r.u8()==0x43 and r.u16()==image;assert r.matrix()==(20,20,0,0,0,0)
        # All four shapes share the identical rectangle edge records.
        assert d[r.at//8:]==bytes.fromhex('0010160c8007081f89c00f58f81f0c801eae1000')
        c,d=ts[sprite];assert c==39 and sprite_frames(d)==[[(shape,256)]]
    c,d=ts[75];assert c==39;frames=sprite_frames(d);assert len(frames)==221
    out=bytearray(b'PBG1'+struct.pack('<H',len(frames)))
    for layers in frames:
        if not 1<=len(layers)<=2:raise ValueError('unexpected layer count')
        out.append(len(layers))
        for cid,alpha in layers:out+=struct.pack('<BH',SPRITES.index(cid),alpha)
    return bytes(out),jpegs,frames

def main():
    p=argparse.ArgumentParser();p.add_argument('swf',type=Path);p.add_argument('output',type=Path);a=p.parse_args();timeline,jpegs,_=extract(a.swf.read_bytes());a.output.mkdir(parents=True,exist_ok=True)
    (a.output/'timeline.pbg').write_bytes(timeline)
    for i,j in enumerate(jpegs):(a.output/f'{i}.jpg').write_bytes(j)
    print(f'221 frames, four original JPEGs, {len(timeline)} timeline bytes, {len(timeline)+sum(map(len,jpegs))} total bytes')
if __name__=='__main__':main()
