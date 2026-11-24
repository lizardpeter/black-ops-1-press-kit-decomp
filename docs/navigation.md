# Navigation and video behavior

Root labels: Welcome 69, Wallpaper 76, Videos 86, Screens 96. Frame 75 installs the menu handlers and shows the mouse. Frame 81 installs eight wallpaper GetURL handlers (preserving the original order 1024x768, 1280x800, 1280x1024, 1440x900, 1680x1050, 1600x1200, 1920x1200, 2560x1440).

Frame 91 at 2724689 installs six trailer buttons; each plays the timeline, sets vid to its button number, and stops music. Frame 95 at 2725445 sets contentPath and starts playback for 1, 2, 4, 5, 6 only; original vid=3 does not change contentPath or start playback in that branch. This is not a claim about the visual availability of the third button. Keep the original behavior in the faithful build. Frame 95 also hides navigation/close/mouse and sets dimensions based on Stage.width with 9/16 aspect ratio. The screen-space offset uses the 1280x720 design reference.

Cursor counter: frame 95 functions at 2726133 and 2726255. Increment counter, compare equality to 50, reset, clear interval, hide mouse, and set Vmouse=0. Timer interval length and host binding are still pending.

The native C++ library models discrete host requests and numeric state, not full timeline integration, Flash rendering or media playback. video_path and video_layout model portions of the frame action, not entire AVM1 functions. They are deliberately excluded from the function ledger.
