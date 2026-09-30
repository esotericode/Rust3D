# Stride 0.6.0 — Highlands and Momentum

Download **Stride-Windows.zip**, extract it, open **Stride-Windows**, then
double-click **Stride.exe**. No installation or Rust toolchain needed.

## New in this build

- A **1440 × 1680 m** landscape, sixteen times the previous map's area, with
  rolling hills, broad open spaces and four mountain masses, with peaks above 200 m.
- **180 additional blocks and 40 additional ramps**, placed with a fixed random
  seed while keeping the original lab, Skyway and an open speed lane clear.
- Five region starting points in **Pause → Highlands / Momentum Routes**.
- Slope following, downhill acceleration, uphill resistance and steep-face
  sliding. Uphill jumps, dives and rollouts retain upward slope momentum.
- Long-jump, dive and rollout chains build speed with diminishing gains and
  **no hard horizontal speed cap**. Landings preserve speed; lighter analog or
  opposite input brakes. Hold Sprint to extend a slide.
- Smooth camera pullback/FOV at speed; HUD peak speed, chain, coordinates and
  slope feedback. Distance-based collision substeps protect fast travel.

Includes the previous textured lighting/soft shadow pass, controller support
with 10% default stick deadzones, remapping, display/frame-cap options, and the
eight-section Skyway course with 46 components and 16 moving platforms.

**Controls:** WASD or left stick moves; Shift or RT/R2 sprints; Space or A/Cross
jumps; Sprint + Jump long jumps; F or X/Square dives; Jump/Dive on the landing
slide rolls out; Esc/Start opens the menu. Reset actions remain menu-only.

The ZIP includes PLAY.txt, LANDSCAPE.md, MOVEMENT.md, COURSE.md and licenses.
MOVEMENT.md records the design references and tuning choices.

## Validation

Release publishing waits for Linux formatting, strict clippy, all tests and
actual Mesa rendering checks, plus Windows tests and a standalone static-CRT
release build. The test suite includes full Skyway traversal at two platform
timings, slope and speed-chain checks, uphill launches, analog precision braking
and thin-wall collision at 300 m/s. Movement runs at a fixed 120 Hz.

This is a playable prototype for 64-bit Windows 10/11 with an OpenGL-capable
graphics driver. Terrain uses heightfield collision and ground adhesion;
arbitrary mesh collision, overhangs and curvature-based crest airtime remain
future work. Hands-on controller and Windows playtesting is still useful for
the final feel adjustments. SHA256.txt accompanies the Windows download.
