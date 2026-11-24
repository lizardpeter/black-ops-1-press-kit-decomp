import functools, tempfile, threading, unittest, urllib.request, urllib.error
from pathlib import Path
from http.server import ThreadingHTTPServer
from serve_web import Handler
class TestRanges(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();Path(self.tmp.name,'test.mp4').write_bytes(bytes(range(100)));Path(self.tmp.name,'test.wasm').write_bytes(b'\0asm')
        self.server=ThreadingHTTPServer(('127.0.0.1',0),functools.partial(Handler,directory=self.tmp.name));self.thread=threading.Thread(target=self.server.serve_forever,daemon=True);self.thread.start();self.url=f'http://127.0.0.1:{self.server.server_port}'
    def tearDown(self):self.server.shutdown();self.server.server_close();self.thread.join();self.tmp.cleanup()
    def request(self,path='/test.mp4',range=None,method='GET'):
        return urllib.request.urlopen(urllib.request.Request(self.url+path,headers={'Range':range} if range else {},method=method))
    def test_ranges(self):
        for query,expected in [('bytes=10-19',bytes(range(10,20))),('bytes=97-',bytes(range(97,100))),('bytes=-4',bytes(range(96,100)))]:
            with self.request(range=query) as r:self.assertEqual(r.status,206);self.assertEqual(r.read(),expected);self.assertEqual(int(r.headers['Content-Length']),len(expected))
        with self.assertRaises(urllib.error.HTTPError) as e:self.request(range='bytes=100-')
        self.assertEqual(e.exception.code,416);self.assertEqual(e.exception.headers['Content-Range'],'bytes */100')
    def test_head_and_wasm(self):
        with self.request(method='HEAD') as r:self.assertEqual(r.headers['Accept-Ranges'],'bytes');self.assertEqual(r.headers['Content-Length'],'100');self.assertEqual(r.read(),b'')
        with self.request('/test.wasm') as r:self.assertEqual(r.headers['Content-Type'],'application/wasm');self.assertEqual(r.read(),b'\0asm')
if __name__=='__main__':unittest.main()
