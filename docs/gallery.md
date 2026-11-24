# Screenshot navigation evidence

Root frame 105 DoAction tag at 3142375. Offsets include the uncompressed 8-byte SWF header.

- 3143432 nextImage: scr < total-1, increment scr before loaded == filesize test; on equality set picture._alpha=1, call picture.loadMovie(image[scr],1), then picture_num().
- 3143575 prevImage: scr > 0, decrement scr, same load/counter sequence, no equality guard.
- 3143696 firstImage: loaded == filesize guard, same load/counter sequence without changing scr.
- 3143802 picture_num: scr+1, " / ", total.

ActionPush double operands at 3143559, 3143598, 3143680, 3143786 are all zero (verified original bytes; AVM1 double word order). Calls to picture_num push argument count zero.

The C++ port models numeric gallery state and emitted host requests; it does not yet implement loading, rendering, XML events, cursor timing or Flash numeric coercion for malformed state. Synthetic tests are not original differential tests.

Frame 101 at 3141214 registers twenty thumbnail handlers. Each plays the timeline and sets scr to 0..19 in order. Frame 105 loadXML at 3143190 checks success; it clears image and description arrays, derives total from xmlNode.childNodes.length, extracts each entry's first child's first child's nodeValue, and calls firstImage without resetting scr. On failure sets content to `file not loaded!`. The XML reader/DOM is a host responsibility; the reconstruction takes extracted text values, not arbitrary malformed Flash XML. Frame 105 has duplicate checkCounter/resetCounter bodies with equivalent operations to those in frame 95.
