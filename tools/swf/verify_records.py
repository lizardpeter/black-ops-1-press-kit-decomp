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

# Owner-approved scope: all function definitions outside Adobe DoInitAction blocks.
expected=set()
for b in ("launcher","presskit"):
    blocks={r["offset"]:r for r in map(json.loads,(root/b/"action_blocks.jsonl").read_text().splitlines())}
    for off,fn in indexes[b].items():
        tag=int(fn["scope"].split("/tag@")[1].split("/")[0])
        if blocks[tag]["tag"]==12:expected.add((b,off))
actual={(r["build"],int(r["offset"])) for r in rows}
if len(expected)!=73 or actual!=expected or len(rows)!=len(expected):
    raise SystemExit(f"authored scope mismatch: expected {len(expected)}; missing {expected-actual}; extra {actual-expected}")
print("Complete authored DoAction function inventory: 73/73; Adobe DoInitAction excluded")
