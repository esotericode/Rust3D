pub mod audio;
pub mod camera;
pub mod controller;
pub mod frame;
pub mod lighting;
pub mod mesh;
pub mod physics;
pub mod renderer;
pub mod ui;

/// The simulation runs independently of rendering at 120 Hz.
pub const FIXED_DT: f32 = 1.0 / 120.0;
