# Media controls

Bytecode verified against R2 original SHA-256 in target.toml. Frame 1 introseq uses payload/intro/IntroSeq.mp4 and starts playback, hides mouse, stops timeline. Its complete callback at 485976 plays the root timeline. The video11 overlay is x-positioned differently (no additional centering term). Frame 56 BG uses the centered video dimensions.

Frame 95 and 105 cursor behavior: mouse move/down sets visible, Vmouse=1 and counter=0; control tick checks counter only if Vmouse==1, then sets visible target y=h-(h-720)/2-100. Hidden target is h-(h-720)/2+50. On the 50th tick the visible target persists until the next tick. Close and completion replace the control callback with stopRoll2 rather than directly changing the timeline. Original callback offsets are in records/avm1_functions.tsv.

Video controls at 2727367: y -= ceil((y-ypos)/5), totalTime and playheadTime rounded before extracting minutes and seconds; each value below 10 gets a literal zero prefix, and the display is headTime/vidTime (no spaces). Gallery controls at 3144409 use the same y easing and compute ymouse from mouse y plus (h-720)/2. Gallery key codes 37 and 39 invoke previous and next respectively.

Loader at 3145040 reads getBytesTotal then getBytesLoaded, shows preloader, on unequal bytes scales bar to 100*loaded/filesize. On equality hides bar and increments alpha by 5 if alpha<500. Preserve the original 500 comparison; Flash host may clamp alpha visually. Native code models the raw request and does not assert Flash display clamping.

Limitations: host events and field side effects are represented as C++ state/requests; no full Flash dynamic coercion or executable integration yet.
