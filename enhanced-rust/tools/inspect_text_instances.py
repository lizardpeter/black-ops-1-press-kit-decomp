#!/usr/bin/env python3
"""Inventory source DefineText/DefineText2 objects placed in the original menu sprites.

This is analysis-only: it does not execute AVM1, modify the SWF, or rasterize artwork.
"""
import argparse
import hashlib
import struct
import zlib
from collections import defaultdict
from pathlib import Path

from extract_background import tags
from extract_controls import Bits, EXPECTED

ROOT_SPRITES=(50,51,53,62,77,80,96,99,111,115,136,157,220)


def placed_character(code,data):
    if code==26 and len(data)>=5 and data[0]&2:
        return struct.unpack_from('<H',data,3)[0]
    if code==70 and len(data)>=6 and data[0]&2:
        return struct.unpack_from('<H',data,4)[0]
    return None


def font_code_tables(all_tags):
    tables={}
    for code,data in all_tags:
        if code not in (13,62) or len(data)<4:
            continue
        font_id=struct.unpack_from('<H',data)[0]
        name_len=data[2]
        pos=3+name_len
        if pos>=len(data):
            continue
        flags=data[pos]
        pos+=1
        if code==62:
            pos+=1  # language code
        width=2 if flags&1 else 1
        codes=[]
        while pos+width<=len(data):
            codes.append(int.from_bytes(data[pos:pos+width],'little'))
            pos+=width
        tables[font_id]=codes
    return tables


def decode_text(code,data,font_tables):
    try:
        r=Bits(data)
        r.u16();r.rect();r.matrix()
        glyph_bits=r.u8();advance_bits=r.u8()
        font_id=None;out=[]
        while r.at//8<len(data):
            flags=r.u8()
            if flags==0:
                break
            if flags&8:
                font_id=r.u16()
            if flags&4:
                for _ in range(4 if code==33 else 3):
                    r.u8()
            if flags&1:
                r.u16()
            if flags&2:
                r.u16()
            if flags&8:
                r.u16()  # font height
            count=r.u8()
            codes=font_tables.get(font_id,[])
            for _ in range(count):
                glyph=r.read(glyph_bits)
                r.read(advance_bits,True)
                cp=codes[glyph] if glyph<len(codes) else None
                out.append(chr(cp) if cp is not None and 0<=cp<=0x10ffff else f'[{glyph}]')
            r.align()
        return ''.join(out).strip()
    except Exception as exc:
        return f'<decode unavailable: {type(exc).__name__}: {exc}>'


def button_characters(data):
    """Return DefineButton2 character references; filtered records are reported as opaque."""
    try:
        r=Bits(data,5)
        out=[]
        while r.at//8<len(data):
            flags=r.u8()
            if flags==0:
                break
            character=r.u16()
            r.u16()  # depth
            r.matrix()
            r.cx()
            if flags&16:
                return out,True
            if flags&32:
                r.u8()
            out.append(character)
        return out,False
    except Exception:
        return [],True


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('swf',type=Path)
    args=parser.parse_args()
    data=args.swf.read_bytes()
    digest=hashlib.sha256(data).hexdigest()
    if digest!=EXPECTED:
        raise SystemExit(f'wrong source SWF: sha256 {digest}')
    raw=b'FWS'+data[3:8]+zlib.decompress(data[8:])
    bits=Bits(raw,8);bits.rect()
    root_tags=list(tags(raw,bits.at//8+4))
    definitions={}
    for code,payload in root_tags:
        if code in (11,13,33,34,39,62):
            definitions[struct.unpack_from('<H',payload)[0]]=(code,payload)
    sprites={i:d for i,(c,d) in definitions.items() if c==39}
    buttons={i:d for i,(c,d) in definitions.items() if c==34}
    texts={i:(c,d) for i,(c,d) in definitions.items() if c in (11,33)}
    font_tables=font_code_tables(root_tags)
    found=defaultdict(set)
    opaque_buttons=set()

    def follow(character,root,path,seen):
        if character in texts:
            found[character].add(path)
        elif character in sprites:
            visit_sprite(character,root,path,seen)
        elif character in buttons:
            visit_button(character,root,path,seen)

    def visit_button(button_id,root,path,seen):
        key=('button',button_id)
        if key in seen:
            return
        seen.add(key)
        children,opaque=button_characters(buttons[button_id])
        if opaque:
            opaque_buttons.add(button_id)
        for index,child in enumerate(children):
            follow(child,root,f'{path}/button#{button_id}[{index}]',seen)

    def visit_sprite(sprite_id,root,path,seen):
        key=('sprite',sprite_id)
        if key in seen:
            return
        seen.add(key)
        for code,payload in tags(sprites[sprite_id],4):
            if code not in (26,70):
                continue
            child=placed_character(code,payload)
            if child is not None:
                follow(child,root,f'{path}/sprite#{sprite_id}',seen)

    # Inspect the main timeline for directly placed text and recurse through the
    # original menu's known top-level reusable sprites.
    frame=1
    for code,payload in root_tags:
        if code in (26,70):
            child=placed_character(code,payload)
            if child is not None:
                follow(child,'root',f'root-frame-{frame}',set())
        elif code==1:
            frame+=1
    for sprite_id in ROOT_SPRITES:
        if sprite_id in sprites:
            visit_sprite(sprite_id,str(sprite_id),f'root-sprite-{sprite_id}',set())

    print(f'Original SWF text inventory: {len(texts)} DefineText objects; {len(found)} referenced by menu/root placements')
    for text_id in sorted(found):
        code,payload=texts[text_id]
        text=decode_text(code,payload,font_tables)
        locations='; '.join(sorted(found[text_id]))
        print(f'  text #{text_id} tag={code}: {text!r}')
        print(f'    placed: {locations}')
    if opaque_buttons:
        print('Button definitions with filters/blend flags not expanded: '+', '.join(map(str,sorted(opaque_buttons))))
    unused=sorted(set(texts)-set(found))
    print(f'Unreferenced text definitions: {len(unused)}')


if __name__=='__main__':
    main()
