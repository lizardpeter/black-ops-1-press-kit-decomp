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
            if vector_menu and c==39:
                from extract_thumbnails import strip
                d2=strip(d2)
                from extract_labels import strip as strip_labels
                d2=strip_labels(d2)
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

def native_art(data,frames):
    """Exact solid geometry and topmost unfiltered static text, not guessed art.
    A cloned sprite strips only the migrated text for eligible root placements;
    original nested/filtered uses of the sprite are left intact.
    """
    from extract_controls import tags as dictionary, render_character, transform
    b=b'FWS'+data[3:8]+zlib.decompress(data[8:]);r=Bits(b,8);r.rect()
    sprites={struct.unpack_from('<H',d)[0]:d for c,d in tags(b,r.at//8+4) if c==39}
    ts=dictionary(data)
    assert not any(i in sprites or i in ts for i in (65000,65001))
    # #77 extends past the stage. The visible portion is exactly this rectangle.
    native={77:[((0,0,0,255),[(0,0),(0,14400),(25600,0),(25600,0),(0,14400),(25600,14400)])]}
    for sid,shape in ((53,23),(80,79),(99,98)):
        # Only convert a single, identity-placed solid batch. Unsupported fill
        # types, multi-fill group alpha, or filtered instances stay in raster.
        rows=[d for c,d in tags(sprites[sid],4) if c==26]
        assert len(rows)==1
        r=Bits(rows[0]);assert r.u8()==6;r.u16();assert r.u16()==shape
        assert r.matrix()==(1,1,0,0,0,0)
        try:batches=render_character(ts,shape)
        except ValueError:continue
        if len(batches)==1 and batches[0][1]:native[sid]=batches
    clones={};extra=bytearray();overlays={}
    for sid,ids,clone in ((50,(47,49),65000),(111,(110,),65001)):
        if not any(cid==sid and a==256 and not flt for frame in frames for cid,_,a,flt,_ in frame):continue
        sprite=sprites[sid];children=[];rest=[]
        for c,d in tags(sprite,4):
            if c==26 and d[0]&2:
                r=Bits(d);flags=r.u8();depth=r.u16();cid=r.u16()
                children.append((depth,cid))
                if cid in ids:
                    assert flags==6
                    m=r.matrix();assert r.at//8==len(d)
                    overlays.setdefault(sid,[]).extend((rgba,[transform(m,p) for p in points]) for rgba,points in render_character(ts,cid) if points)
                    continue
            if c not in (12,59):rest.append(tag(c,d))
        assert tuple(cid for _,cid in sorted(children)[-len(ids):])==ids
        assert overlays.get(sid)
        clones[sid]=clone;extra+=tag(39,struct.pack('<H',clone)+sprite[2:4]+b''.join(rest))
    return native,overlays,clones,bytes(extra)

def build(data,output,jar):
    version,defs,frames=source(data,vector_menu=True);output.mkdir(parents=True,exist_ok=True)
    from extract_menu import pack as pack_menu
    (output/'menu.pkm').write_bytes(pack_menu(data,frames))
    native,overlays,clones,extra=native_art(data,frames)
    # Use a distinct raster variant for partially converted, opaque root sprites.
    variants=sorted({(cid,flt,cid in clones and alpha==256 and not flt) for frame in frames for cid,_,alpha,flt,_ in frame if alpha and not(cid in native and not flt)})
    # Render all reusable source characters in one synthetic, script-free timeline.
    body=bytearray(defs+extra);keys=[]
    for i,(cid,flt,split) in enumerate(variants):
        if i:body+=tag(28,struct.pack('<H',1))
        m=matrix(*ORIGIN);export_id=clones[cid] if split else cid
        if flt:body+=tag(70,struct.pack('<BBHH',6,1,1,export_id)+m+flt)
        else:body+=tag(26,struct.pack('<BHH',6,1,export_id)+m)
        count=4 if cid==51 else 1
        for n in range(count):body+=tag(1);keys.append((cid,flt,split,n))
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
    mesh_slots={};overlay_slots={};meshes=[]
    for table,dest in ((native,mesh_slots),(overlays,overlay_slots)):
        for cid,batches in sorted(table.items()):
            dest[cid]=len(metadata);metadata.append((1,1));meshes.append(batches)
    from extract_thumbnails import extract as extract_thumbnails
    thumbnails=extract_thumbnails(data);jpeg_start=len(metadata);direct={}
    for sid,rows in thumbnails.items():
        direct[sid]=[]
        for cid,x,y,w,h,iw,ih,raw in rows:
            idx=len(metadata);metadata.append((iw,ih));(output/f'{idx}.jpg').write_bytes(raw);direct[sid].append((idx,x,y,w,h))
    packed=bytearray(b'PFG4'+struct.pack('<HH',len(metadata),len(frames)))
    mesh_ids=set(mesh_slots.values())|set(overlay_slots.values())
    for i,(w,h) in enumerate(metadata):packed+=struct.pack('<HHB',w,h,2 if i in mesh_ids else int(i>=jpeg_start))
    for f,frame in enumerate(frames,1):
        placements=[];menu_after=255
        for cid,mat,alpha,flt,born in frame:
            if not alpha:continue
            if cid in mesh_slots and not flt:
                x,y=(0,0) if cid==77 else mat[4:]
                placements.append((mesh_slots[cid],x,y,20,20,alpha));continue
            n=(f-born)%4 if cid==51 else 0
            split=cid in clones and alpha==256 and not flt
            a=assets[cid,flt,split,n]
            if a is None:continue
            if cid in direct and cid!=136:
                assert not flt and alpha==256
                placements.extend((idx,mat[4]+x,mat[5]+y,w,h,alpha) for idx,x,y,w,h in direct[cid])
            idx,x,y=a;w,h=metadata[idx];placements.append((idx,mat[4]+20*x,mat[5]+20*y,w*20,h*20,alpha))
            if cid==136:
                assert not flt and alpha==256
                placements.extend((idx,mat[4]+x,mat[5]+y,w,h,alpha) for idx,x,y,w,h in direct[cid])
            if split:placements.append((overlay_slots[cid],mat[4],mat[5],20,20,256))
            if cid in (136,157,220):menu_after=len(placements)
        packed+=struct.pack('<BB',len(placements),menu_after)
        for a in placements:packed+=struct.pack('<HiiIIH',*a)
    for batches in meshes:
        packed+=struct.pack('<H',len(batches))
        for rgba,vertices in batches:
            assert vertices and len(vertices)%3==0
            packed+=struct.pack('<4BI',*rgba,len(vertices))
            for x,y in vertices:packed+=struct.pack('<ff',x/20,y/20)
    assert len(packed)<=8*1024*1024
    (output/'timeline.pfg').write_bytes(packed)
    print('native vector root characters:',sorted(mesh_slots),'split text overlays:',sorted(overlay_slots))
    print(f'{len(metadata)} reusable cropped textures, {len(keys)} source variants, {len(packed)} timeline bytes; {sum(p.stat().st_size for p in output.iterdir())} bytes total')

def main():
    p=argparse.ArgumentParser();p.add_argument('swf',type=Path);p.add_argument('output',type=Path);p.add_argument('--ffdec-jar',type=Path,required=True);a=p.parse_args();build(a.swf.read_bytes(),a.output,a.ffdec_jar)
if __name__=='__main__':main()
