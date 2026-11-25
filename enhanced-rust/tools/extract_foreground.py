#!/usr/bin/env python3
"""Source-derived foreground display list, original bitmap fills and native triangles.
The runtime asset build uses no offline rasterizer or AVM execution.
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
                if struct.unpack_from('<H',d2)[0]==136:
                    d2=d2[:4]+b''.join(tag(k,v) for k,v in tags(d2,4) if not(k==26 and v[0]&2 and struct.unpack_from('<H',v,3)[0] in (118,132)))
            if vector_menu and c==39 and struct.unpack_from('<H',d2)[0]==51:
                # Retain the animated bitmap button at depth 1, but not the
                # sidebar placement at depth 3, now rendered by the GPU.
                kept=[];removed=0
                for k,v in tags(d2,4):
                    if k==70 and v[0]&2 and struct.unpack_from('<H',v,4)[0]==50:
                        assert struct.unpack_from('<H',v,2)[0]==3;removed+=1;continue
                    if k==28 and v==struct.pack('<H',3):removed+=1;continue
                    kept.append(tag(k,v))
                assert removed==2
                d2=d2[:4]+b''.join(kept)
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
    # Split opaque, unfiltered one-frame sprites into contiguous depth runs.
    # Keep each mask and its clipped children together in the same raster run.
    # Text moved to geometry can now sit *between* raster runs, not just above.
    splits={};extra=bytearray();overlays={};next_id=65000
    for sid,ids in ((62,(55,)),(111,(110,))):
        if not any(cid==sid and a==256 and not flt for frame in frames for cid,_,a,flt,_ in frame):continue
        sprite=sprites[sid]
        assert struct.unpack_from('<H',sprite,2)[0]==1
        parts=[];run=[];kind=None;seen=[];mask_end=0;last_depth=0
        def flush():
            nonlocal next_id,kind,run
            if not run:return
            key=next_id;next_id+=1
            assert key not in sprites and key not in ts
            if kind=='raster':
                extra.extend(tag(39,struct.pack('<H',key)+sprite[2:4]+b''.join(tag(c,d) for c,d in run)+tag(1)+tag(0)))
            else:
                overlays[key]=[batch for batches in run for batch in batches]
            parts.append((kind,key));run=[]
        for c,d in tags(sprite,4):
            if c in (0,1,12,45,59):continue # original SoundStreamHead2 has no drawing effect
            # These three source sprites use only fresh PlaceObject2 records.
            assert c==26
            r=Bits(d);flags=r.u8();depth=r.u16();cid=r.u16()
            assert flags&2 and not flags&1 and depth>last_depth
            last_depth=depth
            if cid in ids:
                assert flags==6 and depth>mask_end
                m=r.matrix();assert r.at//8==len(d)
                batches=[(rgba,[transform(m,p) for p in points]) for rgba,points in render_character(ts,cid) if points]
                assert batches
                seen.append(cid);item=batches;new_kind='mesh'
            else:
                if flags&64:
                    # Mask end is the last field in this source PlaceObject2.
                    mask_end=max(mask_end,struct.unpack_from('<H',d,len(d)-2)[0])
                    assert mask_end>depth
                item=(c,d);new_kind='raster'
            if kind!=new_kind:flush();kind=new_kind
            run.append(item)
        flush()
        assert tuple(seen)==ids and parts
        splits[sid]=parts
    return native,overlays,splits,bytes(extra)

def native_buttons(data):
    from extract_controls import tags as dictionary,render_button
    ts=dictionary(data)
    quit_states=[render_button(ts,115,i) for i in range(3)]
    wallpaper_states=[render_button(ts,118,i) for i in range(3)]
    assert all(quit_states) and not wallpaper_states[0] and all(wallpaper_states[1:])
    raw=b'FWS'+data[3:8]+zlib.decompress(data[8:]);r=Bits(raw,8);r.rect()
    panel=next(d for c,d in tags(raw,r.at//8+4) if c==39 and struct.unpack_from('<H',d)[0]==136)
    placements=[]
    for c,d in tags(panel,4):
        if c!=26 or not d[0]&2 or struct.unpack_from('<H',d,3)[0]!=118:continue
        r=Bits(d);assert r.u8()==38;depth=r.u16();assert r.u16()==118
        m=r.matrix();assert m[:4]==(1,1,0,0)
        assert depth in range(1,16,2)
        placements.append((len(placements)+1,m[4],m[5]))
    assert len(placements)==8
    assert [(x,y) for _,x,y in placements]==[(x,y) for x in (120,3080,6020,8980) for y in (8880,9600)]
    return quit_states,wallpaper_states,placements

def blur_y(flt):
    if not flt:return 0.
    assert len(flt)==11 and flt[:2]==b'\x01\x01' and flt[10]==8
    x,y=struct.unpack_from('<II',flt,2)
    assert x==0 and y<=15*65536
    return y/65536

def nested_sidebar(data):
    raw=b'FWS'+data[3:8]+zlib.decompress(data[8:]);r=Bits(raw,8);r.rect()
    d=next(d for c,d in tags(raw,r.at//8+4) if c==39 and struct.unpack_from('<H',d)[0]==51)
    assert struct.unpack_from('<H',d,2)[0]==4
    f=1;found=[];removed=[];intro=[]
    for c,v in tags(d,4):
        if c==1:f+=1
        elif c==28 and struct.unpack('<H',v)[0]==3:removed.append(f)
        elif c==70:
            r=Bits(v);flags=r.u8();f2=r.u8();depth=r.u16()
            cid=r.u16() if flags&2 else None
            if depth==1:
                assert f2==1 and (flags,cid)==((6,24) if f==1 else (5,None))
                m=r.matrix();assert m[:4]==(1,1,0,0) and blur_y(v[r.at//8:])==15.
                intro.append(m[4:]);continue
            if cid!=50:continue
            assert (flags,f2,depth,f)==(38,1,3,1)
            m=r.matrix();assert m[:4]==(1,1,0,0)
            while r.u8():pass
            found.append((m[4],m[5],blur_y(v[r.at//8:])))
    assert len(found)==1 and removed==[2]
    assert found[0]==(639,14815,15.)
    assert intro==[(0,0),(0,806),(0,1613),(0,2420)]
    return (*found[0],intro)

def build(data,output,jar):
    # Runtime foreground contains original bitmaps and source geometry only.
    # FFDec is retained in the CLI for legacy callers and reference exports,
    # but is not invoked to generate any runtime foreground artwork.
    _,_,frames=source(data);output.mkdir(parents=True,exist_ok=True)
    from extract_menu import pack as pack_menu
    (output/'menu.pkm').write_bytes(pack_menu(data,frames))
    from extract_bitmap_art import extract as direct_art
    bitmaps,overlays,static_roots,menu_strips=direct_art(data)
    from extract_sidebar import extract as sidebar_extract
    sidebar=sidebar_extract(data);nested_x,nested_y,nested_blur,intro=nested_sidebar(data)
    from extract_controls import tags as dictionary,render_character
    overlays[63999]=render_character(dictionary(data),132)
    overlays[77]=[((0,0,0,255),[(0,0),(0,14400),(25600,0),(25600,0),(0,14400),(25600,14400)])]
    for kind,key,value in sidebar:
        if kind=='mesh':overlays[key]=value
    quit_states,wallpaper_states,wallpaper_places=native_buttons(data)
    metadata=[];codecs=[];bitmap_slots={};overlay_slots={};meshes={};button_slots={};button_meshes={};sidebar_slots={};texture_states={}
    def slot(w,h,codec):
        i=len(metadata);metadata.append((w,h));codecs.append(codec);return i
    def png(im,codec=0):
        i=slot(*im.size,codec);im.save(output/f'{i}.png',optimize=True);return i
    for cid,im in sorted(bitmaps.items()):
        if cid==103:
            # Keep the original full JPEG pixels; the renderer clips its UV.
            bitmap_slots[cid]=png(im,4)
            state=((0.,0.,571.,481.),(0.,0.,571/572,1.))
            texture_states[bitmap_slots[cid]]=(0,[state]*3)
        else:bitmap_slots[cid]=png(im)
    # #51 and #53 use the same original #22 image but independent placements.
    intro_slot=png(bitmaps[22])
    for cid,batches in sorted(overlays.items()):
        overlay_slots[cid]=slot(1,1,2);meshes[overlay_slots[cid]]=batches
    for selector,states in [(255,quit_states)]+[(i,wallpaper_states) for i,_,_ in wallpaper_places]:
        button_slots[selector]=slot(1,1,3);button_meshes[button_slots[selector]]=(selector,states)
    for kind,key,value in sidebar:
        if kind=='mesh':continue
        im,states=value;sidebar_slots[key]=png(im,4);texture_states[sidebar_slots[key]]=(key,states)
    strip_slots={}
    for sid,rows in menu_strips.items():
        strip_slots[sid]=[]
        for kind,key,x,y,w,h in rows:
            i=png(bitmaps[key],4);strip_slots[sid].append(i)
            texture_states[i]=(0,[((x/20,y/20,w/20,h/20),(0.,0.,1.,1.))]*3)
    from extract_thumbnails import extract as extract_thumbnails
    direct={}
    for sid,rows in extract_thumbnails(data).items():
        direct[sid]=[]
        for cid,x,y,w,h,iw,ih,raw in rows:
            i=slot(iw,ih,1);(output/f'{i}.jpg').write_bytes(raw);direct[sid].append((i,x,y,w,h))
    packed=bytearray(b'PFG7'+struct.pack('<HH',len(metadata),len(frames)))
    for (w,h),codec in zip(metadata,codecs):packed+=struct.pack('<HHB',w,h,codec)
    for f,frame in enumerate(frames,1):
        placements=[];menu_after=255;filters=[]
        def group(start,alpha,blur):
            if len(placements)>start and (alpha!=256 or blur):filters.append((start,len(placements),alpha,blur))
        def mesh(key,x,y,alpha=256):placements.append((overlay_slots[key],x,y,20,20,alpha))
        for cid,mat,alpha,flt,born in frame:
            if not alpha:continue
            x,y=mat[4:]
            if cid in static_roots:
                start=len(placements)
                for kind,key,dx,dy,w,h in static_roots[cid]:
                    if kind=='mesh':mesh(key,x+dx,y+dy)
                    else:
                        assert all(int(v)==v for v in (dx,dy,w,h))
                        placements.append((bitmap_slots[key],x+int(dx),y+int(dy),int(w),int(h),256))
                group(start,alpha,blur_y(flt));continue
            if cid==77:
                assert not flt;mesh(77,0,0,alpha);continue
            if cid in (50,51):
                blur=blur_y(flt)
                if cid==51:
                    assert not flt
                    n=(f-born)%4;dx,dy=intro[n];w,h=bitmaps[22].size
                    start=len(placements);placements.append((intro_slot,x+dx,y+dy,w*20,h*20,256));group(start,alpha,15.)
                    if n!=0:continue
                    x+=nested_x;y+=nested_y;blur=nested_blur
                start=len(placements)
                for kind,key,_ in sidebar:
                    i=overlay_slots[key] if kind=='mesh' else sidebar_slots[key]
                    placements.append((i,x,y,20,20,256))
                group(start,alpha,blur);continue
            if cid==115:
                assert alpha==256 and not flt;placements.append((button_slots[255],x,y,20,20,256));continue
            assert cid in direct and alpha==256 and not flt
            if cid==136:
                placements.extend((button_slots[i],x+dx,y+dy,20,20,256) for i,dx,dy in wallpaper_places)
                placements.extend((i,x,y,20,20,256) for i in strip_slots[cid])
                mesh(63999,x,y)
                placements.extend((i,x+dx,y+dy,w,h,256) for i,dx,dy,w,h in direct[cid])
            else:
                placements.extend((i,x+dx,y+dy,w,h,256) for i,dx,dy,w,h in direct[cid])
                placements.extend((i,x,y,20,20,256) for i in strip_slots[cid])
            menu_after=len(placements)
        assert len(placements)<=64 and len(filters)<=8
        assert len({p[0] for p in placements})==len(placements)
        packed+=struct.pack('<BBB',len(placements),menu_after,len(filters))
        for a in placements:packed+=struct.pack('<HiiIIH',*a)
        for span in filters:packed+=struct.pack('<BBHf',*span)
    def batches(values):
        packed.extend(struct.pack('<H',len(values)))
        for rgba,vertices in values:
            assert vertices and len(vertices)%3==0
            packed.extend(struct.pack('<4BI',*rgba,len(vertices)))
            for x,y in vertices:packed.extend(struct.pack('<ff',x/20,y/20))
    # The Rust decoder consumes extension records in slot order, not codec
    # order. A clipped bitmap can precede any mesh slot.
    assert set(meshes)=={i for i,c in enumerate(codecs) if c==2}
    assert set(button_meshes)=={i for i,c in enumerate(codecs) if c==3}
    assert set(texture_states)=={i for i,c in enumerate(codecs) if c==4}
    for i,codec in enumerate(codecs):
        if codec==2:batches(meshes[i])
        elif codec==3:
            selector,states=button_meshes[i];packed.append(selector)
            for values in states:batches(values)
        elif codec==4:
            selector,states=texture_states[i];packed.append(selector)
            for bounds,uv in states:packed+=struct.pack('<8f',*bounds,*uv)
    assert len(packed)<=8*1024*1024
    (output/'timeline.pfg').write_bytes(packed)
    print(f'Native foreground: {len(frames)} frames, {len(meshes)} mesh entries, {len(button_meshes)} vector buttons, {len(texture_states)} clipped bitmap entries; no baked runtime sprites')
    print(f'{len(metadata)} source slots, {len(packed)} timeline bytes; {sum(p.stat().st_size for p in output.iterdir())} bytes total')

def main():
    p=argparse.ArgumentParser();p.add_argument('swf',type=Path);p.add_argument('output',type=Path);p.add_argument('--ffdec-jar',type=Path,required=True);a=p.parse_args();build(a.swf.read_bytes(),a.output,a.ffdec_jar)
if __name__=='__main__':main()
