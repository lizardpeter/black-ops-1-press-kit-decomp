#!/usr/bin/env python3
"""Local development server with single-range MP4 streaming, no public bind by default."""
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import argparse, os, re

class Handler(SimpleHTTPRequestHandler):
    extensions_map={**SimpleHTTPRequestHandler.extensions_map,'.wasm':'application/wasm','.mjs':'text/javascript','.mp4':'video/mp4','.mp3':'audio/mpeg'}
    def send_head(self):
        self.range_remaining=None
        path=Path(self.translate_path(self.path))
        if path.is_dir() or not path.is_file():return super().send_head()
        f=path.open('rb');size=os.fstat(f.fileno()).st_size;start,end=0,size-1
        raw=self.headers.get('Range');partial=False
        if raw:
            m=re.fullmatch(r'bytes=(\d*)-(\d*)',raw.strip())
            try:
                if not m or not any(m.groups()) or size==0:raise ValueError()
                a,b=m.groups()
                if a:start=int(a);end=min(int(b),size-1) if b else size-1
                else:
                    n=int(b)
                    if n<=0:raise ValueError()
                    start=max(0,size-n)
                if start>=size or end<start:raise ValueError()
                partial=True
            except ValueError:
                f.close();self.send_response(416);self.send_header('Content-Range',f'bytes */{size}');self.send_header('Content-Length','0');self.end_headers();return None
        self.send_response(206 if partial else 200);self.send_header('Content-Type',self.guess_type(str(path)))
        self.send_header('Accept-Ranges','bytes');self.send_header('Content-Length',str(max(0,end-start+1)))
        self.send_header('Cache-Control','private, max-age=3600' if '/assets/' in self.path else 'no-cache')
        if partial:self.send_header('Content-Range',f'bytes {start}-{end}/{size}')
        self.end_headers();f.seek(start);self.range_remaining=max(0,end-start+1);return f
    def copyfile(self,source,outputfile):
        if self.range_remaining is None:return super().copyfile(source,outputfile)
        while self.range_remaining:
            chunk=source.read(min(65536,self.range_remaining))
            if not chunk:break
            outputfile.write(chunk);self.range_remaining-=len(chunk)

def main():
    p=argparse.ArgumentParser();p.add_argument('--port',type=int,default=8080);p.add_argument('--directory',type=Path,default=Path(__file__).resolve().parents[1]/'web');a=p.parse_args()
    from functools import partial
    with ThreadingHTTPServer(('127.0.0.1',a.port),partial(Handler,directory=str(a.directory))) as s:
        print(f'Local WebGPU player: http://127.0.0.1:{s.server_port}',flush=True);s.serve_forever()
if __name__=='__main__':main()
