#!/usr/bin/env python3
"""Linux local window smoke; requires Xvfb, python-xlib and Pillow.
Use a dedicated X server; never inject input into a real desktop."""
import argparse, os, pathlib, subprocess, tempfile, time
from PIL import Image, ImageChops
from Xlib import X, XK, display
from Xlib.ext import xtest
def main():
    p=argparse.ArgumentParser()
    p.add_argument("exe",type=pathlib.Path)
    p.add_argument("assets",type=pathlib.Path)
    p.add_argument("out",type=pathlib.Path)
    a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        authority=pathlib.Path(tmp)/"Xauthority";authority.touch()
        # Xvfb chooses an unused display; dedicated socket with no other clients.
        read_fd,write_fd=os.pipe()
        xvfb=subprocess.Popen(["Xvfb","-displayfd",str(write_fd),"-screen","0","1280x720x24","-ac","-nolisten","tcp"],pass_fds=(write_fd,),stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        os.close(write_fd);app=None;d=None
        try:
            disp=os.read(read_fd,32).decode().strip();os.close(read_fd)
            if not disp:raise RuntimeError("Xvfb failed")
            env={**os.environ,"DISPLAY":":"+disp,"XAUTHORITY":str(authority),"XDG_RUNTIME_DIR":tmp}
            os.environ.update({k:env[k] for k in ("DISPLAY","XAUTHORITY")})
            with (a.out/"run.log").open("w") as log:
                app=subprocess.Popen([str(a.exe.resolve()),str(a.assets.resolve())],env=env,stdout=log,stderr=log)
                d=display.Display();root=d.screen().root
                time.sleep(2)
                def snap(name,ref,origin=(0,0)):
                    if app.poll() is not None:raise RuntimeError("app exited")
                    im=root.get_image(0,0,1280,720,X.ZPixmap,0xffffffff)
                    got=Image.frombytes("RGB",(1280,720),im.data,"raw","BGRX");got.save(a.out/(name+".png"))
                    source=Image.open(a.assets/"art"/ref).convert("RGBA")
                    expected=Image.new("RGB",(1280,720));expected.paste(source,origin,source)
                    if ImageChops.difference(got,expected).getbbox() is not None:raise AssertionError(name+" pixel mismatch")
                    print(name,"exact pixel match")
                def click(x,y):
                    xtest.fake_input(d,X.MotionNotify,x=x,y=y);xtest.fake_input(d,X.ButtonPress,1);xtest.fake_input(d,X.ButtonRelease,1);d.sync();time.sleep(.2)
                def key(name):
                    k=d.keysym_to_keycode(XK.string_to_keysym(name));xtest.fake_input(d,X.KeyPress,k);xtest.fake_input(d,X.KeyRelease,k);d.sync()
                snap("launcher","launcher/1.png",(390,210))
                click(450,420);key("Return");time.sleep(4)
                snap("welcome","presskit/75.png")
                click(250,270);time.sleep(1)
                snap("videos","presskit/91.png")
        finally:
            if d:d.close()
            if app:app.terminate();app.wait(timeout=10)
            xvfb.terminate();xvfb.wait(timeout=10)
if __name__=="__main__":main()
