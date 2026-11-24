#!/usr/bin/env python3
"""Check authored AVM1 function identity against fresh, generated function indexes."""
import csv,json,sys
from pathlib import Path
root=Path(sys.argv[1])
indexes={}
for b in ("launcher","presskit"):
    p=root/b/"functions.jsonl"
    indexes[b]={f["offset"]:f for f in map(json.loads,p.read_text().splitlines())}
    blocks=list(map(json.loads,(root/b/"action_blocks.jsonl").read_text().splitlines()))
    print(b,len(indexes[b]),"function definitions;",len(blocks),"blocks;",sum("parse_error" in x for x in blocks),"block parse errors")
    if not indexes[b]: raise SystemExit("missing function index")
with open("records/avm1_functions.tsv",newline="") as f:
    rows=list(csv.DictReader(f,delimiter="\t"))
for r in rows:
    fn=indexes[r["build"]].get(int(r["offset"]))
    if fn is None or fn["body_size"]!=int(r["body_size"]):
        raise SystemExit("record not supported by fresh index: "+str(r))
    if fn["name"] and fn["name"]!=r["name"]:
        raise SystemExit("named function mismatch: "+str(r))
print("Verified",len(rows),"original function identities (offset and body length; not behavioral equivalence)")
