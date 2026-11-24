#!/usr/bin/env python3
"""Prepare original art and embedded MP3s outside git, no runtime SWF execution."""
import argparse,hashlib,re,shutil,subprocess,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).parent/"swf"))
from analyze import unpack,u16,u32
def sound_export(path,out):
    b=unpack(path.read_bytes());p=8+(5+4*(b[8]>>3)+7)//8+4
    out.mkdir(parents=True,exist_ok=True);ids=[]
    while p+2<=len(b):
        h=u16(b,p);p+=2;code,n=h>>6,h&63
        if n==63:n=u32(b,p);p+=4
        e=p+n
        if e>len(b):raise ValueError("invalid tag size")
        if code==14:
            i=u16(b,p);fmt=b[p+2]>>4
            if fmt!=2:raise ValueError("unexpected original sound format")
            # DefineSound MP3: ID, flags, sample count, signed seek samples, MP3 data.
            data=b[p+9:e]
            if not data:raise ValueError("empty MP3")
            (out/f"{i}.mp3").write_bytes(data);ids.append(i)
        if code==0:break
        p=e
    if sorted(ids)!=[1,2,28,52,154]:raise ValueError("sound identity mismatch")
def normalize(source,dest,count):
    dest.mkdir(parents=True,exist_ok=True);found={}
    for p in source.rglob("*.png"):
        m=re.fullmatch(r"(?:frame[_ -]?)?0*(\d+)",p.stem,re.I)
        if not m:raise ValueError("unknown exported frame name "+str(p))
        i=int(m[1])
        if i in found:raise ValueError("duplicate frame")
        found[i]=p
    if set(found)!=set(range(1,count+1)):raise ValueError("frame identity mismatch "+str(sorted(found)))
    for i,p in found.items():shutil.copyfile(p,dest/f"{i}.png")
def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("original",type=Path);ap.add_argument("output",type=Path)
    ap.add_argument("--reference",type=Path,required=True)
    a=ap.parse_args()
    original=a.original.resolve();output=a.output.resolve()
    if original==output or original in output.parents:raise ValueError("use a separate output directory")
    swf=original/"COD7-DVD.swf"
    if hashlib.sha256(swf.read_bytes()).hexdigest()!="1521acdea7d20691da8d917c0e306ce9e39208406377c836b68bc457400c8d8e":raise ValueError("wrong main SWF")
    for name,count in (("launcher",1),("presskit",105)):
        normalize(a.reference/name,output/"art"/name,count)
    sound_export(swf,output/"art"/"sounds")
    shutil.copytree(original/"payload",output/"payload",dirs_exist_ok=True)
    print("Prepared 106 original artwork frames, 5 original embedded MP3s and original payload")
if __name__=="__main__":main()
