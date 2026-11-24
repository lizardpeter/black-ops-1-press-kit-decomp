#!/usr/bin/env python3
from pathlib import Path
import sys,hashlib
from PIL import Image
from prepare_player_assets import normalize
def main():
    src,dest=map(Path,sys.argv[1:3])
    for name,count in (("foreground",105),("background",221),("video_controls",1),("gallery_controls",1)):
        normalize(src/name,dest/name,count)
        for p in (dest/name).glob("*.png"):
            with Image.open(p) as im:
                if im.size!=(1280,720):raise ValueError("unexpected stage size "+str(p))
        print(name,count,"frames")
    im=Image.open(dest/"foreground/75.png").convert("RGBA")
    if im.getchannel("A").getextrema()[0]!=0:raise ValueError("foreground has no transparency")
    for name in ("video_controls","gallery_controls"):
        p=dest/name/"1.png"
        im=Image.open(p).convert("RGBA")
        lo,hi=im.getchannel("A").getextrema()
        if lo!=0 or hi==0:raise ValueError("invalid control layer "+name)
        im.crop((0,0,880,90)).save(p)
    hashes={hashlib.sha256(p.read_bytes()).hexdigest() for p in (dest/"background").glob("*.png")}
    if len(hashes)<4:raise ValueError("background export appears static")
    print("Background distinct images:",len(hashes))
if __name__=="__main__":main()
