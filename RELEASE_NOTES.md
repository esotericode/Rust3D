# Stride 0.7.0 — Movement Feel and VSync

Download **Stride-Windows.zip**, extract it, open **Stride-Windows**, then
double-click **Stride.exe**. On Linux, download **Stride-Linux-x86_64.tar.gz**,
extract it and run `Stride-Linux/Play.sh`. No installation or Rust toolchain
needed.

## Changed controls

- **Long jump is now Crouch + Jump while moving**: Ctrl + Space, or LT + A on a
  controller. Crouch is a new, remappable binding; saved 0.6 preferences gain
  it on LT, or on the first free button.
- **Jumping while sprinting keeps its full height** and your speed. Before, any
  sprinting jump was a low long jump.

## Movement

- **Wall slides:** hold toward a wall in the air to slide down it at a steady
  speed, then press Jump at any point to kick off. Alternate walls to climb.
- **Crest airtime:** fast enough over a crest that falls away faster than
  gravity can follow, you leave the ground on a natural arc.
- **Smooth analog braking:** how much earned speed you keep now blends with
  stick strength and turn angle. A slightly lighter push no longer drops you
  from full speed to a walk, and pressing only a strafe key at speed no longer
  brakes like reversing.
- **Steerable belly slides**, and a rollout that lands early on raised ground
  no longer locks out your next jump.
- Steep faces can no longer be climbed by mashing jump, a mid-air dive no
  longer cancels a fast fall, and the character no longer sinks into block
  edges.

## Display, sound and presentation

- **VSync option, on by default**, to prevent tearing. Changes apply the next
  time the game starts. The default frame cap is now 240 FPS, which only binds
  if a driver forces VSync off.
- 25 Highlands ramps no longer float above the slopes, and ramp walls are lit
  correctly. Terrain and mountain rock lose their repeating ripple pattern.
- Footsteps follow the legs and no longer buzz at high speed. Wall-kick and
  dive sounds no longer warble, and the beacon chime no longer clicks.
- The camera keeps you in frame during long falls.
- HUD text needs a third of the geometry, and `, ! ?` now display.

**Controls:** WASD or left stick moves; Shift or RT/R2 sprints; Space or A/Cross
jumps; Ctrl or LT/L2 + Jump long jumps; F or X/Square dives; Jump/Dive on the
landing slide rolls out; hold into a wall to slide; Esc/Start opens the menu.

The ZIP includes PLAY.txt, LANDSCAPE.md, MOVEMENT.md, COURSE.md and licenses.
MOVEMENT.md records the design references and tuning choices.

## Validation

Release publishing waits for Linux formatting, strict clippy, all tests and
actual Mesa rendering checks, plus Windows tests and a standalone static-CRT
release build. The test suite traverses the full Skyway course at two platform
timings using crouch long jumps, and covers crest airtime, wall slides,
steep-face hops, ledge slips, analog smoothness, ramp grounding and sound
sweeps. Movement runs at a fixed 120 Hz.

This is a playable prototype for 64-bit Windows 10/11 and Linux x86_64 with an
OpenGL-capable graphics driver. Hands-on controller playtesting is still useful
for the final feel adjustments. SHA256.txt accompanies the Windows download.
