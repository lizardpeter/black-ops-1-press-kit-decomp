# Application scope

Owner decision 2026-11-24: no Adobe Flash decompilation. 36,002 historical projector functions and 425 bundled Adobe AVM1 functions are outside scope. The 73 application definitions (3 launcher, 70 main) are the authored function denominator, not a whole-application completion percentage. Four identical background callbacks named Play are recovered at offsets 2377744, 2378254, 2378764, 2379274. Each clears nInterval and plays the sprite. Surrounding frame code stops and schedules Play with interval 2000 ms; source bytes were checked directly. Native model uses an elapsed-time accumulator with independent sprite playback, no AVM1 VM.

Still required: inline frame actions, original artwork, hit regions, media backend, end-to-end visual and behavioral comparison. No generic Adobe runtime implementation.
