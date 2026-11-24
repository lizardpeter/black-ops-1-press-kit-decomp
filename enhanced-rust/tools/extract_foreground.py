#!/usr/bin/env python3
"""Source-derived foreground display list. Rasterize reusable characters, not stage frames.
Original root translations and opacity are retained at full twip precision.
Nested artwork/blur variants use offline artwork rasterization, never AVM execution.
"""
import argparse, hashlib, struct, subprocess, tempfile, zlib
from pathlib import Path
from PIL import Image
from extract_controls import Bits, EXPECTED
from extract_background import tags
DEFINITIONS={2,6,7,8,10,11,13,14,17,20,21,22,23,32,33,34,35,36,37,39,46,48,56,60,62,69,73,74,75,76,77,78,83,84,87,88,90,91}
IDS={50,51,53,62,77,80,96,99,111,115,136,157,220}
SKIP={11,75,159,191,222,240}
ORIGIN=(200,100)
SIZE=(1280,2300)
def tag(c,d=b''):
    n=len(d);return (struct.pack('<H',c<<6|n) if n<63 else struct.pack('<HI',c<<6|63,n))+d

def bits(*fields):
    s=''.join(format(v&((1<<n)-1),f'0{n}b') for v,n in fields if n);s+='0'*(-len(s)%8);return int(s,2).to_bytes(len(s)//8,'big')

def matrix(x,y):return bits((0,1),(0,1),(17,5),(x*20,17),(y*20,17))
def rect(w,h):return bits((17,5),(0,17),(w*20,17),(0,17),(h*20,17))
def strip_sprite(d):return d[:4]+b''.join(tag(c,strip_sprite(v) if c==39 else v) for c,v in tags(d,4) if c not in (12,59))

def source(data, vector_menu=False):
    if hashlib.sha256(data).hexdigest()!=EXPECTED:raise ValueError('wrong original SWF')
    b=b'FWS'+data[3:8]+zlib.decompress(data[8:]);r=Bits(b,8);assert r.rect()==[0,25600,0,14400];header=r.at//8+4;ts=list(tags(b,header));defs=[];frames=[];live={}
    for c,d in ts:
        if c in DEFINITIONS:
            if c==39:
                from extract_menu import strip_buttons
                replacement=strip_buttons(d) if vector_menu else None
                d2=strip_sprite(d) if replacement is None else replacement
            else:d2=d
            defs.append(tag(c,d2))
        if c in (26,70):
            r=Bits(d);flags=r.u8();f2=r.u8() if c==70 else 0;depth=r.u16();old=live.get(depth) if flags&1 else None
            cid,mat,alpha,flt,born=old or (None,(1,1,0,0,0,0),256,b'',len(frames)+1)
            if flags&2:cid=r.u16();born=len(frames)+1
            if flags&4:mat=r.matrix()
            if flags&8:
                mul,add=r.cx();assert mul[:3]==[256]*3 and add==[0]*4;alpha=mul[3]
            if flags&16:r.u16()
            if flags&32:
                while r.u8():pass
            if flags&64:raise ValueError('root masks unsupported')
            if f2&1:
                tail=d[r.at//8:];assert tail==b'\0' or len(tail)==11 and tail[:2]==b'\1\1';flt=tail if tail!=b'\0' else b''
            if f2&~3:raise ValueError('unsupported root placement')
            if cid not in IDS|SKIP:raise ValueError(f'unknown root character {cid}')
            if cid in IDS and (mat[:4]!=(1,1,0,0) or not 0<=alpha<=256):raise ValueError('unexpected root transform')
            live[depth]=(cid,mat,alpha,flt,born)
        elif c==28:live.pop(struct.unpack('<H',d)[0])
        elif c==1:frames.append([live[k] for k in sorted(live) if live[k][0] in IDS])
    assert len(frames)==105
    return b[3:4],b''.join(defs),frames

def build(data,output,jar):
    version,defs,frames=source(data,vector_menu=True);output.mkdir(parents=True,exist_ok=True)
    from extract_menu import pack as pack_menu
    (output/'menu.pkm').write_bytes(pack_menu(data,frames))
    variants=sorted({(cid,flt) for frame in frames for cid,_,alpha,flt,_ in frame if cid!=77 and alpha})
    # Render all reusable source characters in one synthetic, script-free timeline.
    body=bytearray(defs);keys=[]
    for i,(cid,flt) in enumerate(variants):
        if i:body+=tag(28,struct.pack('<H',1))
        m=matrix(*ORIGIN)
        if flt:body+=tag(70,struct.pack('<BBHH',6,1,1,cid)+m+flt)
        else:body+=tag(26,struct.pack('<BHH',6,1,cid)+m)
        count=4 if cid==51 else 1
        for n in range(count):body+=tag(1);keys.append((cid,flt,n))
    body+=tag(0);header=rect(*SIZE)+struct.pack('<HH',24*256,len(keys));swf=b'FWS'+version+struct.pack('<I',8+len(header)+len(body))+header+body
    with tempfile.TemporaryDirectory() as tmp:
        tmp=Path(tmp);(tmp/'characters.swf').write_bytes(swf);(tmp/'frames').mkdir()
        subprocess.run(['java','-Djava.awt.headless=true','-jar',str(jar),'-ignorebackground','-format','frame:png','-export','frame',str(tmp/'frames'),str(tmp/'characters.swf')],check=True,stdout=subprocess.DEVNULL)
        paths=sorted((tmp/'frames').rglob('*.png'),key=lambda p:int(p.stem));assert len(paths)==len(keys),(len(paths),len(keys))
        assets={};metadata=[];digests={}
        for key,p in zip(keys,paths):
            im=Image.open(p).convert('RGBA');assert im.size==SIZE;box=im.getchannel('A').getbbox()
            if box is None:assets[key]=None;continue
            if box[0]==0 or box[1]==0 or box[2]==SIZE[0] or box[3]==SIZE[1]:raise ValueError(f'character clipped at export: {key} {box}')
            im=im.crop(box);digest=hashlib.sha256(im.tobytes()+struct.pack('<II',*im.size)).digest()
            if digest not in digests:
                idx=len(metadata);digests[digest]=idx;im.save(output/f'{idx}.png',optimize=True);metadata.append(im.size)
            assets[key]=(digests[digest],box[0]-ORIGIN[0],box[1]-ORIGIN[1])
    # Original #77 is an opaque black rectangle extending beyond all four stage edges.
    black=len(metadata);metadata.append((1,1));Image.new('RGBA',(1,1),(0,0,0,255)).save(output/f'{black}.png')
    packed=bytearray(b'PFG2'+struct.pack('<HH',len(metadata),len(frames)))
    for w,h in metadata:packed+=struct.pack('<HH',w,h)
    for f,frame in enumerate(frames,1):
        placements=[]
        for cid,mat,alpha,flt,born in frame:
            if not alpha:continue
            if cid==77:placements.append((black,0,0,25600,14400,alpha));continue
            n=(f-born)%4 if cid==51 else 0
            a=assets[cid,flt,n]
            if a is None:continue
            idx,x,y=a;w,h=metadata[idx];placements.append((idx,mat[4]+20*x,mat[5]+20*y,w*20,h*20,alpha))
        packed+=struct.pack('<B',len(placements))
        for a in placements:packed+=struct.pack('<HiiIIH',*a)
    (output/'timeline.pfg').write_bytes(packed)
    print(f'{len(metadata)} reusable cropped textures, {len(keys)} source variants, {len(packed)} timeline bytes; {sum(p.stat().st_size for p in output.iterdir())} bytes total')

def main():
    p=argparse.ArgumentParser();p.add_argument('swf',type=Path);p.add_argument('output',type=Path);p.add_argument('--ffdec-jar',type=Path,required=True);a=p.parse_args();build(a.swf.read_bytes(),a.output,a.ffdec_jar)
if __name__=='__main__':main()
