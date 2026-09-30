# Stride / Rust3D

A custom Rust 3D engine and third-person movement playground. Run, jump,
long jump, dive, roll out, and climb by kicking between walls. Version 0.6 adds
the **1440 x 1680 metre Highlands**, sixteen times the previous map's area,
with rolling hills, open speed lanes, mountains above 200 m and seeded blocks
and ramps. Slope gravity and uncapped, diminishing movement gains let you build
momentum through long jumps, dives and rollouts. **Skyway** offers eight
progressively harder sections with 46 ordered components,
16 moving platforms, lifts, fixed rest decks and section practice. The original
**360 x 420 metre** yard remains inside Highlands, with eleven ramps, block gardens, the
14.3 metre tower and seven-beacon movement lab.

## Play on Windows

Download **Stride-Windows.zip** from the
[latest GitHub release](https://github.com/esotericode/Rust3D/releases/latest).
Development builds are also available in the successful
[Build and test Stride workflow](https://github.com/esotericode/Rust3D/actions/workflows/build.yml).
Extract it, open `Stride-Windows`, and **double-click `Stride.exe`**.
No installation or Rust setup required. Needs 64-bit Windows 10/11 and an
OpenGL-capable graphics driver. There are no external assets or bundled DLLs.
CI builds with the static Microsoft C runtime.

| Control | Action |
| --- | --- |
| WASD / arrows | Camera-relative movement |
| Shift | Sprint |
| Space | Tap for a hop, hold for a high jump; press promptly at a wall to kick |
| Shift + Space while moving | Long jump |
| Q / E or right mouse drag | Orbit camera |
| Mouse wheel | Zoom |
| F | Dive; press Jump or Dive during the landing slide to roll out |
| C | Recenter camera |
| Enter | Confirm menu selection |
| F1 / F2 / F11 / Esc | Controls / options / fullscreen / pause |

| Standard gamepad control | Action |
| --- | --- |
| Left stick | Analog walk/run |
| Right stick | Orbit camera |
| A / Cross | Jump / wall kick / confirm |
| RT / R2 | Sprint; add jump for a long jump |
| X / Square | Dive; Jump or Dive rolls out after landing |
| Y / Triangle | Recenter camera |
| Start / Menu | Pause / resume |
| Select / View | Controls |
| D-pad or left stick | Navigate menus |

Respawn at checkpoint and Restart Course are **pause-menu actions**. Restart
opens a confirmation dialog with Cancel selected. There are no gameplay reset
buttons or keys; the old X, R, and Enter reset shortcuts are removed.

Dive on the ground or once during an airborne jump. It preserves forward
momentum, lands in a short slide, and can chain into a rollout hop. A jump
pressed shortly before landing is buffered. Without another press, the slide
recovers automatically; hold Sprint to extend it. Under low ceilings, it stays low and permits slow
movement until there is enough room to stand.

Gamepads use gilrs mappings and support hotplugging. The default circular
deadzone is **10% on both sticks**; Controller Settings lets you change each
stick independently from 0% to 30%, remap four gameplay actions, and toggle
vibration. Duplicate bindings swap rather than triggering two actions.
Input outside the deadzone is rescaled continuously, preserving slow walking.
Use a controller supported by Windows Gaming Input; Xbox-compatible controllers
are the primary target. Other devices depend on driver and mapping support.

Wall kicks require a fresh jump press during the first 120 ms of wall contact,
with 35 ms of grace after leaving contact. Alternate walls to climb. Beacons
activate in order and save your respawn position. Falling resets automatically.
The timer begins on movement; best times last for the current session.

## Highlands and momentum in 0.6

Choose **Pause → Highlands / Momentum Routes** for five starting regions:
Long Run, Rolling Basin, Ridge Descent, Mountain Pass and Block Fields. The
larger landscape adds 180 seeded blocks and 40 ramps, while keeping the old
routes and an open 550 m speed lane clear. Region travel sets a respawn point.

Slope following keeps feet on the rendered terrain. Gravity adds speed downhill
and spends it uphill; steep faces slide, and uphill launches retain upward
momentum. Long jumps, dives and rollouts add speed with diminishing returns,
without a hard horizontal cap. Strong forward input and air coasting retain
earned speed. Lighter analog input or opposite input brakes for precise pads;
fast turns require wider arcs. Speed, peak and recent chain appear in the HUD.
The camera smoothly widens and pulls back at speed. Collision is subdivided by
distance to protect high-speed movement against thin obstacles.

See [LANDSCAPE.md](LANDSCAPE.md) for regions and [MOVEMENT.md](MOVEMENT.md) for
design references, movement rules and testing/tuning targets.

## Graphics in 0.5

The renderer now uses per-pixel sun lighting, roughness-dependent highlights,
2048-pixel soft directional shadow maps, cool sky fill and a warm sun. Blocks,
platform bodies and the robot have tessellated rounded bevels with smooth
normals. Concrete, courtyard pavers, painted metal and rubber have distinct
responses to light. An embedded seamless texture supplies albedo variation,
small normal-map surface relief and roughness variation; mipmaps filter it at
a distance. Material coordinates travel with animated objects. The sky has a
horizon gradient and a sun disc, and distance fog is evaluated per pixel.

The pass changes rendered geometry and shading. Gameplay collision retains the
authored block/ramp dimensions, and course markers keep their clear colors.
Fine surface relief uses shading rather than GPU tessellation or displaced
collision. Shadows cover a 104 m square around the character and fade at its
edge; distant structures still receive sunlight and sky fill. This is direct
lighting with approximate sky fill, rather than full global illumination.
The executable generates its materials at startup and needs no texture files.

## Skyway

Open **Esc / Start → Skyway / Section Practice → Start Full Skyway Course**.
You can also walk up the wide entrance ramp east of the original lab. Practice
any of the eight sections directly from that menu; practice times do not enter
the session best. Return to Playground takes you back to the original lab.

- Amber platforms move; mint edges mark fixed surfaces; violet decks save checkpoints.
- Route arrows, numbered upcoming targets and dotted motion rails show the way.
- Board a shuttle, ride it toward the destination, then jump near its endpoint.
- Lifts and diagonal shuttles carry you; their velocity is inherited when you jump
  or walk off. Movement and platform rendering share interpolation.
- Each section ends on a fixed checkpoint deck. Falling below the section returns
  you there and replays that section. Respawning keeps platform phases and the
  timer; restarting resets both. All platforms freeze while paused.
- The timer counts actual unpaused time from your first movement/action, including
  retries. Full-course best times last for the session.

See [COURSE.md](COURSE.md) for the route and its design intent.

## Resolution and frame rate

Open **F2**, or choose **Options** in the pause menu. Select a row and use
left/right (or mouse clicks) to change it, then select **Apply and Save**.
Keyboard, mouse, and controller navigation are supported.

- Render resolutions: 960 x 540, 1280 x 720, 1600 x 900, 1920 x 1080, 2560 x 1440.
- Frame caps: 30, 60, 90, 120, 144, 165, 240 FPS, or uncapped.
- Windowed/fullscreen mode.
- Camera: 25–200% sensitivity, invert vertical look, optional automatic alignment.
- Controller: separate movement/camera deadzones, jump/dive/sprint/recenter
  bindings, optional vibration. Menu A/B and Start retain their standard roles.
- Sound: effects volume, including mute. All effects are synthesized in Rust.

The default is **1280 x 720 at a 60 FPS cap**. Render resolution controls the
actual offscreen game image. Windowed mode requests that window size; desktop
DPI scaling can make its physical size differ. Fullscreen scales the game image
to the display with aspect-preserving letterboxing. Rendering uses 4x MSAA when
the GPU supports multisample resolve, with a single-sample fallback.

The cap limits rendered frames with a clock-based limiter. GPU performance and
driver overrides may produce a lower actual FPS, shown in the HUD and Options.
Movement always uses a **fixed 120 Hz simulation** and catches up between
rendered frames. Platforms, the character and camera blend between completed simulation
states for smooth rendering when the frame and simulation rates differ. Changing the cap does not change movement constants or game
speed at supported frame rates. Settings are saved under
`%LOCALAPPDATA%/Rust3D/stride-options.cfg` on Windows, or the XDG configuration
directory on Linux.

## Develop

Install stable Rust. Linux also needs `libudev-dev`, `libasound2-dev`, and `pkg-config` for input/audio
(for example, `sudo apt-get install libudev-dev libasound2-dev pkg-config`), then:

```sh
cargo run
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --all -- --check
```

On Windows, `cargo build --release` produces `target/release/stride.exe`.
CI also packages a standalone MSVC build. On Linux, use a desktop with
OpenGL drivers. For a headless rendering check:

```sh
cargo build --locked
xvfb-run -a env LIBGL_ALWAYS_SOFTWARE=1 target/debug/stride --smoke-test --screenshot stride.ppm
```

## Engine

**miniquad** handles native window/input and low-level GPU access; **glam**
provides vector/matrix math; **gilrs** handles gamepad devices and mappings; **cpal** supplies native audio
output. The Rust engine synthesizes and mixes its own movement sounds. If no
audio output is available, play continues silently.
There is no prebuilt game or physics engine.
The repository owns the fixed 120 Hz simulation, character collision,
procedural geometry, shaders, HUD font, camera, and game logic.

| File | Responsibility |
| --- | --- |
| `src/engine/physics.rs` | Acceleration, air control, jump buffering, coyote time, wall kicks and collision |
| `src/engine/terrain.rs` | Shared heightfield triangles, smooth terrain meshes and region starting points |
| `src/engine/renderer.rs` | World, shadow, sky and UI passes, textures, buffers and capture |
| `src/engine/lighting.rs` / `src/engine/shaders/` | Stable sun shadows, procedural material texture and lighting shaders |
| `src/engine/mesh.rs` | Procedural geometry |
| `src/engine/camera.rs` | Third-person follow/orbit and obstruction checks |
| `src/engine/controller.rs` | Gamepad hotplugging, remapping, radial deadzones and vibration |
| `src/engine/frame.rs` | Render frame pacing |
| `src/engine/audio.rs` | Procedural sound synthesis, mixer and output |
| `src/engine/ui.rs` | Built-in font and HUD geometry |
| `src/world.rs` | Collision surfaces, platform updates, playground and beacons |
| `src/course.rs` | Deterministic platform paths, authored Skyway route and checkpoints |
| `src/app.rs` | Game loop, input, course logic and animated robot |
| `src/menu.rs` / `src/settings.rs` | Options, menu navigation and saved preferences |

This prototype uses a variable-height upright capsule against heightfield terrain, blocks, ramps and
kinematic platforms. Arbitrary mesh collision and an editor are future work.
Tests cover platform carrying through reversals, inherited jump momentum,
lift landings, head pinches, checkpoints, practice rules, dive/rollout,
controller settings, camera behavior, slope gravity, high-speed chains,
uphill launches, terrain agreement, thin-wall collision and existing routes. A
lookahead test pilot completes all 46 course components using only gameplay
inputs, from two different starting phases. CI renders the course and menus,
and builds/tests on Windows. Physical controller, vibration and Windows audio
playtesting remain hardware checks.
All game geometry and UI glyphs are generated in code.

To publish a release, push a branch named `release/v<package-version>` containing
the reviewed build and RELEASE_NOTES.md. Its build workflow publishes that
exact commit and Windows package only after the Linux and Windows jobs pass.
The publish job has write access to repository contents; other jobs are read-only.

## License

MIT for Rust3D. See THIRD_PARTY.txt for dependencies.
