#!/usr/bin/env python3
import argparse, pathlib, subprocess, re, json
p=argparse.ArgumentParser();p.add_argument("--slangc",default="slangc");p.add_argument("--out",type=pathlib.Path,required=True);a=p.parse_args()
source=pathlib.Path(__file__).resolve().parents[1]/"shaders/composite.slang"
a.out.mkdir(parents=True,exist_ok=True)
entries={}
for stage,entry in (("vertex","vertexMain"),("fragment","fragmentMain")):
    out=a.out/(stage+".wgsl")
    subprocess.run([a.slangc,str(source),"-target","wgsl","-entry",entry,"-stage",stage,"-o",str(out)],check=True)
    match=re.search(r"@"+stage+r"\b[\s\S]*?\bfn\s+(\w+)",out.read_text())
    if not match:raise SystemExit("no WGSL entry point")
    entries[stage]=match.group(1)
(a.out/"entries.json").write_text(json.dumps(entries))
print(entries)
