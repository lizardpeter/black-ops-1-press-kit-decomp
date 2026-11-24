# Display-list and render reference

tools/swf/display_list.py exports source-derived root and sprite timelines, character definitions, depth-sorted placements, affine matrices, RGBA color transforms, clipping depths, and move/removal operations. It verifies 105 main frames, 1 launcher frame, original stage dimensions, exact placed menu instances and three root transforms. Coordinates in exported matrices are original integer twips and 16.16 coefficients. The remaining filter/blend/clip-action data is flagged, not interpreted. Static snapshots are not a full Flash execution model.

reference-render workflow exports static original SWF artwork frames using pinned FFDec as a visual comparison artifact (not a runtime dependency). Original-derived image assets remain out of git. FFDec frame renders are not proof of original interactive runtime behavior. Future faithful renderer should consume original vector shapes, images, fonts, depth masks and event state rather than treat the reference screenshots as an implementation.

The first native timeline host is implemented in src/timeline.cpp. Native rendering and hit-testing are not yet implemented.
