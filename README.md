# Stride / Rust3D

A custom Rust 3D engine and third-person movement playground. Run, jump,
long jump, and climb by kicking between walls. Five ordered beacons make a
timed route through ramps, blocks, a gap, and a wall-kick lane.

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
| Space | Jump / wall kick while touching a wall |
| Shift + Space while moving | Long jump |
| Q / E or right mouse drag | Orbit camera |
| Mouse wheel | Zoom |
| R | Respawn at checkpoint |
| Enter | Restart course / resume pause |
| F1 / F11 / Esc | Help / fullscreen / pause |

Wall kicks require a fresh Space press; alternate walls to climb. Beacons
activate in order and save your respawn position. Falling resets automatically.
The timer begins on movement; best times last for the current session.

## Develop

Install stable Rust, then:

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
provides vector/matrix math. There is no prebuilt game or physics engine.
The repository owns the fixed 120 Hz simulation, character collision,
procedural geometry, shaders, HUD font, camera, and game logic.

| File | Responsibility |
| --- | --- |
| `src/engine/physics.rs` | Acceleration, air control, jump buffering, coyote time, wall kicks and collision |
| `src/engine/renderer.rs` | GPU pipeline, buffers, lighting/grid/fog shaders and capture |
| `src/engine/mesh.rs` | Procedural geometry |
| `src/engine/camera.rs` | Third-person follow/orbit and obstruction checks |
| `src/engine/ui.rs` | Built-in font and HUD geometry |
| `src/world.rs` | Solids, ramp surfaces, level and beacons |
| `src/app.rs` | Game loop, input, course logic and animated robot |

This first version uses an upright box character collider and axis-separated
collision against static geometry. Arbitrary mesh collision, moving platforms,
audio, gamepads, saved settings, and an editor are future work. Deterministic
integration tests check movement; CI renders an actual frame with Mesa and
builds/tests on Windows. All game geometry and UI glyphs are generated in code.

## License

MIT.
