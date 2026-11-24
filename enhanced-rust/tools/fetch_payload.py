#!/usr/bin/env python3
"""Fetch and verify the complete public BO1 press-kit payload at build time."""
from __future__ import annotations
import argparse
import concurrent.futures
import hashlib
import json
import os
import re
import time
from pathlib import Path, PurePosixPath
from urllib.error import URLError, HTTPError
from urllib.parse import quote
from urllib.request import Request, urlopen

DEFAULT_BASE = "https://r2.houseofkublai.com/bo1-press-kit/payload/"
CHUNK = 1024 * 1024

def validate_manifest(doc):
    files = doc.get("files")
    if not isinstance(files, list) or not files:
        raise ValueError("manifest files must be a non-empty list")
    seen = set()
    total = 0
    for item in files:
        rel = item.get("path", "")
        parts = rel.split("/") if isinstance(rel, str) else []
        if not parts or any(p in ("", ".", "..") for p in parts) or "\\" in rel or rel.startswith("/"):
            raise ValueError("unsafe payload path: " + repr(rel))
        if rel in seen:
            raise ValueError("duplicate payload path: " + rel)
        seen.add(rel)
        if not isinstance(item.get("size"), int) or item["size"] <= 0:
            raise ValueError("invalid payload size: " + rel)
        if not re.fullmatch(r"[0-9a-fA-F]{32}", item.get("etag", "")):
            raise ValueError("invalid R2 ETag: " + rel)
        total += item["size"]
    if doc.get("total_bytes") != total:
        raise ValueError(f"manifest total mismatch: declared {doc.get('total_bytes')}, calculated {total}")
    return total

def target_for(root: Path, rel: str) -> Path:
    parts = rel.split("/")
    if not parts or any(p in ("", ".", "..") for p in parts) or "\\" in rel or rel.startswith("/"):
        raise ValueError("unsafe payload path: " + repr(rel))
    base = root.resolve()
    target = base.joinpath(*parts).resolve()
    if base not in target.parents:
        raise ValueError("payload path escaped destination: " + rel)
    return target

def verify_file(path: Path, item: dict) -> bool:
    if not path.is_file() or path.stat().st_size != item["size"]:
        return False
    digest = hashlib.md5(usedforsecurity=False)
    with path.open("rb") as stream:
        while chunk := stream.read(CHUNK):
            digest.update(chunk)
    return digest.hexdigest().lower() == item["etag"].lower()

def download_one(item: dict, root: Path, base_url: str, retries: int = 3) -> tuple[str, int]:
    rel = item["path"]
    target = target_for(root, rel)
    target.parent.mkdir(parents=True, exist_ok=True)
    if verify_file(target, item):
        return rel, item["size"]
    url = base_url.rstrip("/") + "/" + quote(rel, safe="/")
    temp = target.with_name(target.name + ".part")
    last_error = None
    for attempt in range(1, retries + 1):
        try:
            digest = hashlib.md5(usedforsecurity=False)
            count = 0
            request = Request(url, headers={"User-Agent": "black-ops-1-press-kit-build/1", "Accept-Encoding": "identity"})
            with urlopen(request, timeout=180) as response, temp.open("wb") as output:
                if response.status != 200:
                    raise RuntimeError(f"{rel}: unexpected HTTP {response.status}")
                response_etag = (response.headers.get("ETag") or "").strip('"')
                if response_etag and response_etag.lower() != item["etag"].lower():
                    raise RuntimeError(f"{rel}: R2 ETag changed ({response_etag})")
                while chunk := response.read(CHUNK):
                    output.write(chunk)
                    digest.update(chunk)
                    count += len(chunk)
            if count != item["size"]:
                raise RuntimeError(f"{rel}: size mismatch, expected {item['size']}, got {count}")
            if digest.hexdigest().lower() != item["etag"].lower():
                raise RuntimeError(f"{rel}: MD5/ETag mismatch")
            os.replace(temp, target)
            return rel, count
        except Exception as exc:
            last_error = exc
            try:
                temp.unlink(missing_ok=True)
            except OSError:
                pass
            if attempt < retries:
                time.sleep(attempt * 2)
    raise RuntimeError(f"failed to fetch {rel} after {retries} attempts: {last_error}") from last_error

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--destination", type=Path, required=True)
    parser.add_argument("--base-url", default=DEFAULT_BASE)
    parser.add_argument("--workers", type=int, default=6)
    args = parser.parse_args()
    doc = json.loads(args.manifest.read_text(encoding="utf-8"))
    total = validate_manifest(doc)
    if not 1 <= args.workers <= 16:
        parser.error("--workers must be between 1 and 16")
    args.destination.mkdir(parents=True, exist_ok=True)
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as pool:
        futures = [pool.submit(download_one, item, args.destination, args.base_url) for item in doc["files"]]
        completed = 0
        for future in concurrent.futures.as_completed(futures):
            rel, size = future.result()
            completed += 1
            print(f"[{completed:02d}/{len(futures)}] verified {rel} ({size:,} bytes)", flush=True)
    actual = sum(target_for(args.destination, item["path"]).stat().st_size for item in doc["files"])
    if actual != total:
        raise SystemExit(f"payload directory total mismatch: expected {total}, got {actual}")
    print(f"Verified {len(doc['files'])} original payload files: {actual:,} bytes ({actual / 1048576:.2f} MiB).", flush=True)

if __name__ == "__main__":
    main()
