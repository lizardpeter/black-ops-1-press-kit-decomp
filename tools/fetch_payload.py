#!/usr/bin/env python3
"""Stream each recorded R2 original; verify object ETag/length, compute SHA-256.
No original content is written to git. Optional local destination is outside repo.
"""
import argparse,csv,hashlib,re,sys,urllib.parse,urllib.request
from pathlib import Path,PurePosixPath
from fetch_originals import config
def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--manifest",type=Path,required=True)
    ap.add_argument("--dest",type=Path)
    args=ap.parse_args()
    base=config()["r2_public"].rstrip("/")+"/"
    args.manifest.parent.mkdir(parents=True,exist_ok=True)
    with open("records/originals.tsv",newline="") as f:
        entries=list(csv.DictReader(f,delimiter="\t"))
    results=[]
    for row in entries:
        key=row["key"];parts=PurePosixPath(key).parts
        if not parts or parts[0]!="bo1-press-kit" or ".." in parts or "\\" in key: raise ValueError("unsafe key "+key)
        req=urllib.request.Request(base+urllib.parse.quote(key),headers={"User-Agent":"t5-press-kit-analysis"})
        sha=hashlib.sha256();md5=hashlib.md5(usedforsecurity=False);length=0
        out=None;tmp=None
        if args.dest:
            out=args.dest.joinpath(*parts[1:])
            out.parent.mkdir(parents=True,exist_ok=True)
            tmp=out.with_name(out.name+".partial")
        try:
            with urllib.request.urlopen(req,timeout=120) as r:
                tag=r.headers.get("ETag","").strip('"')
                f=tmp.open("wb") if tmp else None
                try:
                    while chunk:=r.read(1024*1024):
                        length+=len(chunk);sha.update(chunk);md5.update(chunk)
                        if length>int(row["size"]): raise ValueError("size overflow "+key)
                        if f: f.write(chunk)
                finally:
                    if f:f.close()
            expected=row["etag"]
            if length!=int(row["size"]): raise ValueError("size mismatch "+key)
            if re.fullmatch(r"[0-9a-fA-F]{32}",expected):
                if md5.hexdigest()!=expected.lower(): raise ValueError("MD5/ETag mismatch "+key)
            elif tag!=expected: raise ValueError("ETag mismatch "+key)
            if tmp:tmp.replace(out)
        finally:
            if tmp:tmp.unlink(missing_ok=True)
        results.append((key,length,expected,sha.hexdigest()))
        print(key,length,sha.hexdigest(),flush=True)
    with args.manifest.open("w",newline="") as f:
        w=csv.writer(f,delimiter="\t",lineterminator="\n")
        w.writerow(("key","size","etag","sha256"));w.writerows(results)
    print("Verified",len(results),"files",sum(r[1] for r in results),"bytes")
if __name__=="__main__":main()
