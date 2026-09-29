# Stride / Rust3D

A custom Rust 3D engine and third-person movement playground. Run, jump,
long jump, and climb by kicking between walls. Version 0.2 adds gamepad input,
refined movement, a 112 x 112 metre playground, six ramps, a 14.3 metre spiral
tower, seven timed beacons, and saved display/input options.

## Play on Windows

Download **Stride-Windows.zip** from the latest successful
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
| R | Respawn at checkpoint |
| Enter | Restart course / confirm menu selection |
| F1 / F2 / F11 / Esc | Controls / options / fullscreen / pause |

| Standard gamepad control | Action |
| --- | --- |
| Left stick | Analog walk/run |
| Right stick | Orbit camera |
| A / Cross | Jump / wall kick / confirm |
| RT or B / Circle | Sprint; add jump for a long jump |
| X / Square | Respawn |
| Y / Triangle | Recenter camera |
| Start / Menu | Pause / resume |
| Select / View | Controls |
| D-pad or left stick | Navigate menus |

Gamepads use gilrs mappings and support hotplugging. The default circular
deadzone is **10% on both sticks**; Options lets you change it from 0% to 30%.
Input outside the deadzone is rescaled continuously, preserving slow walking.
Use a controller supported by Windows Gaming Input; Xbox-compatible controllers
are the primary target. Other devices depend on driver and mapping support.

Wall kicks require a fresh jump press during the first 120 ms of wall contact,
with 35 ms of grace after leaving contact. Alternate walls to climb. Beacons
activate in order and save your respawn position. Falling resets automatically.
The timer begins on movement; best times last for the current session.

## Resolution and frame rate

Open **F2**, or choose **Options** in the pause menu. Select a row and use
left/right (or mouse clicks) to change it, then select **Apply and Save**.
Keyboard, mouse, and controller navigation are supported.

- Render resolutions: 960 x 540, 1280 x 720, 1600 x 900, 1920 x 1080, 2560 x 1440.
- Frame caps: 30, 60, 90, 120, 144, 165, 240 FPS, or uncapped.
- Windowed/fullscreen mode and stick deadzone.

The default is **1280 x 720 at a 60 FPS cap**. Render resolution controls the
actual offscreen game image. Windowed mode requests that window size; desktop
DPI scaling can make its physical size differ. Fullscreen scales the game image
to the display with aspect-preserving letterboxing. Rendering uses 4x MSAA when
the GPU supports multisample resolve, with a single-sample fallback.

The cap limits rendered frames with a clock-based limiter. GPU performance and
driver overrides may produce a lower actual FPS, shown in the HUD and Options.
Movement always uses a **fixed 120 Hz simulation** and catches up between
rendered frames. Changing the cap does not change movement constants or game
speed at supported frame rates. Settings are saved under
`%LOCALAPPDATA%/Rust3D/stride-options.cfg` on Windows, or the XDG configuration
directory on Linux.

## Develop

Install stable Rust. Linux also needs `libudev-dev` and `pkg-config` for gamepad
support (for example, `sudo apt-get install libudev-dev pkg-config`), then:

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
provides vector/matrix math; **gilrs** handles gamepad devices and mappings.
There is no prebuilt game or physics engine.
The repository owns the fixed 120 Hz simulation, character collision,
procedural geometry, shaders, HUD font, camera, and game logic.

| File | Responsibility |
| --- | --- |
| `src/engine/physics.rs` | Acceleration, air control, jump buffering, coyote time, wall kicks and collision |
| `src/engine/renderer.rs` | GPU pipeline, buffers, lighting/grid/fog shaders and capture |
| `src/engine/mesh.rs` | Procedural geometry |
| `src/engine/camera.rs` | Third-person follow/orbit and obstruction checks |
| `src/engine/controller.rs` | Gamepad hotplugging, mappings and radial deadzone |
| `src/engine/frame.rs` | Render frame pacing |
| `src/engine/ui.rs` | Built-in font and HUD geometry |
| `src/world.rs` | Solids, ramp surfaces, level and beacons |
| `src/app.rs` | Game loop, input, course logic and animated robot |
| `src/menu.rs` / `src/settings.rs` | Options, menu navigation and saved preferences |

This first version uses an upright box character collider and axis-separated
collision against static geometry. Arbitrary mesh collision, moving platforms,
audio, vibration, custom controller rebinding, and an editor are future work.
Tests cover deadzones, analog movement, hop/high-jump behavior, prompt/late wall
kicks, the tower route, frame-rate independence, menu navigation, and settings.
CI renders menus and the tower at multiple resolutions and builds/tests on
Windows. Physical controller testing remains a separate hardware check.
All game geometry and UI glyphs are generated in code.

## License

MIT for Rust3D. See THIRD_PARTY.txt for dependencies.
