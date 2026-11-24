#!/usr/bin/env python3
"""Export authored SWF placements and frame snapshots, not a raster renderer."""
import json,sys
from pathlib import Path
from analyze import unpack,u16,u32,string

class Bits:
    def __init__(self,b,p,end): self.b=b;self.pos=p*8;self.end=end*8
    def take(self,n,signed=False):
        if self.pos+n>self.end: raise ValueError("bit field outside tag")
        value=0
        for _ in range(n):
            value=(value<<1)|((self.b[self.pos//8]>>(7-self.pos%8))&1)
            self.pos+=1
        return value-(1<<n) if signed and n and value&(1<<(n-1)) else value
    def end_byte(self): return (self.pos+7)//8

def rect(b,p,end):
    r=Bits(b,p,end);n=r.take(5)
    return [r.take(n,True) for _ in range(4)],r.end_byte()

def matrix(b,p,end):
    r=Bits(b,p,end);sx=sy=65536;r0=r1=0
    if r.take(1):
        n=r.take(5);sx,sy=r.take(n,True),r.take(n,True)
    if r.take(1):
        n=r.take(5);r0,r1=r.take(n,True),r.take(n,True)
    n=r.take(5);x,y=r.take(n,True),r.take(n,True)
    return {"scale_x_16":sx,"scale_y_16":sy,"rotate_skew_0_16":r0,"rotate_skew_1_16":r1,"x_twips":x,"y_twips":y},r.end_byte()

def color(b,p,end):
    r=Bits(b,p,end);add,mul,n=r.take(1),r.take(1),r.take(4)
    m=[r.take(n,True) for _ in range(4)] if mul else [256]*4
    a=[r.take(n,True) for _ in range(4)] if add else [0]*4
    return {"multiply":m,"add":a},r.end_byte()

def inspect(data):
    b=unpack(data);bounds,p=rect(b,8,len(b));p+=4
    definitions=[];placements=[];frames=[];errors=[]
    def scan(p,end,scope,depth=0):
        if depth>64: raise ValueError("sprite nesting")
        frame=1;display={}
        while p+2<=end:
            off=p;h=u16(b,p);p+=2;code,n=h>>6,h&63
            if n==63:n=u32(b,p);p+=4
            e=p+n
            if e>end:raise ValueError("tag outside timeline")
            if code in (2,6,7,10,11,14,20,21,22,32,33,34,35,36,37,39,46,48,60,75,83,84,87,90,91):
                definition={"id":u16(b,p),"code":code,"offset":off,"size":n,"scope":scope}
                if code in (2,22,32,83):definition["bounds_twips"]=rect(b,p+2,e)[0]
                definitions.append(definition)
            if code==39:scan(p+4,e,f"{scope}/{u16(b,p)}",depth+1)
            elif code in (26,70):
                try:
                    f=b[p];f2=b[p+1] if code==70 else 0;q=p+(2 if code==70 else 1)
                    d=u16(b,q);q+=2
                    update={"scope":scope,"frame":frame,"offset":off,"depth":d,"move":bool(f&1),"flags":f,"flags2":f2}
                    if code==70 and (f2&8 or (f2&16 and f&2)):update["class_name"],q=string(b,q,e)
                    if f&2:update["character"]=u16(b,q);q+=2
                    if f&4:update["matrix"],q=matrix(b,q,e)
                    if f&8:update["color"],q=color(b,q,e)
                    if f&16:update["ratio"]=u16(b,q);q+=2
                    if f&32:update["name"],q=string(b,q,e)
                    if f&64:update["clip_depth"]=u16(b,q);q+=2
                    if q>e:raise ValueError("placement fields outside tag")
                    update["clip_actions"]=bool(f&128)
                    if q<e:update["remaining_bytes"]=e-q
                    placements.append(update)
                    state=dict(display.get(d,{})) if f&1 else {}
                    state.update(update);display[d]=state
                except (ValueError,IndexError) as ex:
                    errors.append({"scope":scope,"offset":off,"error":str(ex)})
            elif code==28:
                d=u16(b,p);display.pop(d,None)
                placements.append({"scope":scope,"frame":frame,"offset":off,"depth":d,"remove":True})
            elif code==1:
                frames.append({"scope":scope,"frame":frame,"layers":[dict(display[d]) for d in sorted(display)]});frame+=1
            elif code==0:return
            p=e
    scan(p,len(b),"root")
    return {"bounds_twips":bounds,"definitions":definitions,"placements":placements,"frames":frames,"errors":errors,
            "coverage":"PlaceObject2/3, RemoveObject2, root and sprite frames, affine/color transforms; filter and clip-action payloads not interpreted"}

def verify(obj,build):
    expected=[0,10000,0,6000] if build=="launcher" else [0,25600,0,14400]
    if obj["bounds_twips"]!=expected or obj["errors"]:raise ValueError("layout extraction error")
    root=[r for r in obj["frames"] if r["scope"]=="root"]
    if len(root)!=(1 if build=="launcher" else 105):raise ValueError("root frame count")
    if build=="presskit":
        def named(sprite,prefix):
            return {r["name"]:r for r in obj["placements"] if r["scope"]=="root/"+str(sprite) and r.get("name","").startswith(prefix)}
        for sprite,prefix,ids in [(136,"wp",range(1,9)),(157,"trl_",[1,2,4,5,6]),(220,"scrn_",range(1,21))]:
            found=named(sprite,prefix)
            if set(found)!={prefix+str(i) for i in ids}:raise ValueError("menu placement mismatch")
        for frame,character,name in [(81,136,"wallpaper"),(91,157,"videos"),(101,220,"screens")]:
            layer=next(x for x in root[frame-1]["layers"] if x.get("name")==name)
            if layer["character"]!=character or layer["matrix"]["x_twips"]!=10537 or layer["matrix"]["y_twips"]!=860:
                raise ValueError("root placement mismatch")
    return len(root)

def main():
    source,out,build=Path(sys.argv[1]),Path(sys.argv[2]),sys.argv[3]
    obj=inspect(source.read_bytes());count=verify(obj,build)
    out.mkdir(parents=True,exist_ok=True)
    for key in ("definitions","placements","frames"):
        with (out/(key+".jsonl")).open("w",encoding="utf8") as f:
            for r in obj.pop(key):f.write(json.dumps(r,ensure_ascii=False)+"\n")
    (out/"layout.json").write_text(json.dumps(obj,indent=2)+"\n")
    print(build,count,"root frames, verified placement identities and transforms")
if __name__=="__main__":main()
