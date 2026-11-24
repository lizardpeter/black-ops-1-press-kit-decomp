# Initial R2 byte inspection

56 nonempty original objects, 429,953,645 bytes; prefix bo1-press-kit/. No retail game executable in this prefix.
Windows projector PE machine 0x14c, base 0x400000, entry RVA 0x30f03b (decimal 3207227), timestamp 1233627407. Projector trailer magic 0xFA123456, little-endian length 45496, embedded CWS at file offset 4343680. Embedded SHA-256 matches the macOS launcher SWF exactly.
Mac fat magic CAFEBABE, two slices (PowerPC and i386); Info.plist reports Flash Player 10.0.2.54.
Both SWFs CWS v10, FileAttributes=16 (AVM1), no root DoABC. Launcher 25 FPS, 1 frame, 56597 bytes uncompressed. Main 24 FPS, 105 frames, 3147604 bytes uncompressed. Root labels: Welcome=69, Wallpaper=76, Videos=86, Screens=96. Initial intro file: payload/intro/IntroSeq.mp4. Main root frame 105 uses payload/screens/screens.xml (20 image entries). Frame 95 references five movie paths. FLVPlayback and mx.video component initialization blocks are third-party library code.

Launcher frame 1 action block at offset 1322, three function definitions at 1529, 1656, 1781. Stage scaleMode noScale, align TL; FSCommand showmenu false. Frame callback positions shell (width/2 - 640,height/2 - 360). Yes loads COD7-DVD.swf in shell, requests fullscreen, hides no/yes/txt/launch. No requests quit.

Gallery picture_num at offset 3143802, body 37 bytes: scrctrl.pos = (scr + 1) + " / " + total. Other gallery handlers recorded but not promoted until their operands and branches are confirmed.

Offsets are in the uncompressed SWF including its header; they are not native VAs. SHA-256 in target.toml was calculated from actual R2 bytes. Ghidra and automated bytecode exports remain to be verified by CI.
