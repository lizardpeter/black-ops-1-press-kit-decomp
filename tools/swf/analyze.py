#!/usr/bin/env python3
"""Deterministic AVM1 timeline and action-block export; FFDec handles other script containers."""
import argparse, collections, hashlib, json, struct, zlib
from pathlib import Path

def u16(b, p): return struct.unpack_from("<H", b, p)[0]
def u32(b, p): return struct.unpack_from("<I", b, p)[0]
def string(b, p, limit):
    e = b.index(0, p, limit)
    return b[p:e].decode("utf-8", errors="replace"), e + 1
def unpack(data):
    if len(data)<8: raise ValueError("truncated SWF")
    size=u32(data,4)
    if size<8 or size>64*1024*1024: raise ValueError("unsupported size")
    if data[:3]==b"CWS":
        d=zlib.decompressobj()
        body=d.decompress(data[8:],size-7)
        if not d.eof or d.unused_data or d.unconsumed_tail: raise ValueError("bad compressed stream")
    elif data[:3]==b"FWS": body=data[8:]
    else: raise ValueError("unsupported SWF signature")
    if len(body)+8!=size: raise ValueError("incorrect uncompressed length")
    return b"FWS"+data[3:8]+body

def actions(b, start, limit, scope, functions, records, depth=0):
    if depth>64: raise ValueError("function nesting exceeds limit")
    p=start
    while p<limit:
        at=p; op=b[p]; p+=1
        if op==0: break
        n=0
        if op>=128:
            if p+2>limit: raise ValueError("short action header")
            n=u16(b,p); p+=2
        end=p+n
        if end>limit: raise ValueError("action outside block")
        row={"offset":at,"opcode":f"{op:02x}","payload_hex":b[p:end].hex(),"scope":scope}
        if op in (0x9b,0x8e):
            name,q=string(b,p,end)
            count=u16(b,q); q+=2
            if op==0x8e: q+=3
            params=[]
            for _ in range(count):
                if op==0x8e: q+=1
                param,q=string(b,q,end);params.append(param)
            if q+2!=end: raise ValueError("unexpected function header")
            size=u16(b,q)
            if end+size>limit: raise ValueError("function body outside block")
            fn={"offset":at,"body_offset":end,"body_size":size,"scope":scope,"name":name,"parameters":params,"sha256":hashlib.sha256(b[end:end+size]).hexdigest()}
            functions.append(fn);row["function"]=fn
            records.append(row)
            actions(b,end,end+size,f"{scope}/fn@{at}",functions,records,depth+1)
            p=end+size
        else:
            records.append(row);p=end

def inspect(data):
    b=unpack(data)
    p=8+(5+4*(b[8]>>3)+7)//8
    rate=u16(b,p)/256; frames=u16(b,p+2);p+=4
    tags=[];blocks=[];functions=[];instructions=[];labels=[];flags=[]
    def scan(p,limit,scope,depth=0):
        if depth>64: raise ValueError("sprite nesting exceeds limit")
        frame=1
        while p+2<=limit:
            at=p;h=u16(b,p);p+=2;code=h>>6;n=h&63
            if n==63:
                if p+4>limit: raise ValueError("short tag header")
                n=u32(b,p);p+=4
            end=p+n
            if end>limit: raise ValueError("tag outside sprite")
            tags.append({"offset":at,"code":code,"size":n,"scope":scope,"frame":frame})
            if code==1: frame+=1
            elif code==43: labels.append({"offset":at,"scope":scope,"frame":frame,"label":string(b,p,end)[0]})
            elif code==69: flags.append(u32(b,p))
            elif code==39: scan(p+4,end,f"{scope}/{u16(b,p)}",depth+1)
            elif code in (12,59):
                start=p+(2 if code==59 else 0)
                block={"offset":at,"action_offset":start,"size":end-start,"scope":scope,"frame":frame,"tag":code}
                if code==59: block["sprite_id"]=u16(b,p)
                blocks.append(block)
                # Record malformed/unsupported blocks rather than silently dropping them.
                try: actions(b,start,end,f"{scope}/tag@{at}",functions,instructions)
                except (ValueError,struct.error,IndexError) as e: block["parse_error"]=str(e)
            p=end
            if code==0: return
    scan(p,len(b),"root")
    return {"sha256":hashlib.sha256(data).hexdigest(),"version":b[3],"uncompressed_size":len(b),"frame_rate":rate,"frame_count":frames,"file_attributes":flags,"tag_counts":dict(sorted(collections.Counter(t["code"] for t in tags).items())),"labels":labels,"action_blocks":blocks,"functions":functions,"instructions":instructions,"tags":tags,"coverage":"DoAction and DoInitAction including sprite timelines and nested function bodies; clip/button/try/with actions need FFDec or further parser work"}

def main():
    ap=argparse.ArgumentParser();ap.add_argument("input",type=Path);ap.add_argument("output",type=Path);args=ap.parse_args()
    obj=inspect(args.input.read_bytes());args.output.mkdir(parents=True,exist_ok=True)
    for key in ("tags","action_blocks","functions","instructions"):
        with (args.output/(key+".jsonl")).open("w",encoding="utf-8") as f:
            for row in obj.pop(key): f.write(json.dumps(row,ensure_ascii=False)+"\n")
    (args.output/"metadata.json").write_text(json.dumps(obj,indent=2)+"\n")
if __name__=="__main__": main()
