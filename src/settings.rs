use std::{io, path::PathBuf};

pub const RESOLUTIONS: [(u32, u32); 5] = [
    (960, 540),
    (1280, 720),
    (1600, 900),
    (1920, 1080),
    (2560, 1440),
];
pub const FRAME_CAPS: [Option<u32>; 8] = [
    Some(30),
    Some(60),
    Some(90),
    Some(120),
    Some(144),
    Some(165),
    Some(240),
    None,
];
pub const BUTTON_NAMES: [&str; 8] = [
    "A / CROSS",
    "B / CIRCLE",
    "X / SQUARE",
    "Y / TRIANGLE",
    "LB / L1",
    "RB / R1",
    "LT / L2",
    "RT / R2",
];
pub const BINDING_NAMES: [&str; 5] = [
    "JUMP",
    "DIVE / ROLLOUT",
    "SPRINT",
    "RECENTER",
    "CROUCH / LONG JUMP",
];

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub resolution: usize,
    pub frame_cap: usize,
    pub fullscreen: bool,
    pub deadzone_percent: u32,
    pub look_deadzone_percent: u32,
    pub camera_sensitivity: u32,
    pub invert_y: bool,
    pub auto_camera: bool,
    pub bindings: [usize; 5],
    pub vibration: bool,
    pub volume: u32,
    /// Applied when the window opens, so a change takes effect on restart.
    pub vsync: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            resolution: 1,
            // VSync paces frames to the display; the 240 cap only applies if a
            // driver overrides VSync off.
            frame_cap: 6,
            fullscreen: false,
            deadzone_percent: 10,
            look_deadzone_percent: 10,
            camera_sensitivity: 100,
            invert_y: false,
            auto_camera: false,
            bindings: [0, 2, 7, 3, 6],
            vibration: true,
            volume: 60,
            vsync: true,
        }
    }
}
impl Settings {
    pub fn size(&self) -> (u32, u32) {
        RESOLUTIONS[self.resolution]
    }
    pub fn cap(&self) -> Option<u32> {
        FRAME_CAPS[self.frame_cap]
    }
    pub fn deadzone(&self) -> f32 {
        self.deadzone_percent as f32 / 100.
    }
    pub fn look_deadzone(&self) -> f32 {
        self.look_deadzone_percent as f32 / 100.
    }
    pub fn sensitivity(&self) -> f32 {
        self.camera_sensitivity as f32 / 100.
    }
    pub fn bind(&mut self, action: usize, button: usize) {
        if let Some(other) = self.bindings.iter().position(|b| *b == button) {
            self.bindings[other] = self.bindings[action];
        }
        self.bindings[action] = button;
    }
    pub fn encode(&self) -> String {
        format!("resolution={}\nframe_cap={}\nfullscreen={}\ndeadzone={}\nlook_deadzone={}\ncamera_sensitivity={}\ninvert_y={}\nauto_camera={}\nbindings={},{},{},{},{}\nvibration={}\nvolume={}\nvsync={}\n",
            self.resolution,self.frame_cap,self.fullscreen,self.deadzone_percent,self.look_deadzone_percent,
            self.camera_sensitivity,self.invert_y,self.auto_camera,self.bindings[0],self.bindings[1],self.bindings[2],self.bindings[3],self.bindings[4],self.vibration,self.volume,self.vsync)
    }
    pub fn decode(text: &str) -> Self {
        let mut s = Self::default();
        // Old files had one deadzone for both sticks.
        let mut separate_look = false;
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let number = value.parse::<usize>().ok();
            match key {
                "resolution" => {
                    if let Some(v) = number.filter(|v| *v < RESOLUTIONS.len()) {
                        s.resolution = v;
                    }
                }
                "frame_cap" => {
                    if let Some(v) = number.filter(|v| *v < FRAME_CAPS.len()) {
                        s.frame_cap = v;
                    }
                }
                "fullscreen" => {
                    if let Ok(v) = value.parse() {
                        s.fullscreen = v;
                    }
                }
                "deadzone" => {
                    if let Some(v) = number.filter(|v| *v <= 30 && v % 5 == 0) {
                        s.deadzone_percent = v as u32;
                    }
                }
                "look_deadzone" => {
                    if let Some(v) = number.filter(|v| *v <= 30 && v % 5 == 0) {
                        s.look_deadzone_percent = v as u32;
                        separate_look = true;
                    }
                }
                "camera_sensitivity" => {
                    if let Some(v) = number.filter(|v| (25..=200).contains(v) && v % 25 == 0) {
                        s.camera_sensitivity = v as u32;
                    }
                }
                "invert_y" => {
                    if let Ok(v) = value.parse() {
                        s.invert_y = v;
                    }
                }
                "auto_camera" => {
                    if let Ok(v) = value.parse() {
                        s.auto_camera = v;
                    }
                }
                "vibration" => {
                    if let Ok(v) = value.parse() {
                        s.vibration = v;
                    }
                }
                "vsync" => {
                    if let Ok(v) = value.parse() {
                        s.vsync = v;
                    }
                }
                "volume" => {
                    if let Some(v) = number.filter(|v| *v <= 100 && v % 10 == 0) {
                        s.volume = v as u32;
                    }
                }
                "bindings" => {
                    let mut values: Vec<usize> =
                        value.split(',').filter_map(|v| v.parse().ok()).collect();
                    // Files from before the crouch binding list four actions;
                    // crouch takes LT when free, otherwise the first free button.
                    if values.len() == 4 {
                        let free = std::iter::once(6)
                            .chain(0..BUTTON_NAMES.len())
                            .find(|b| !values.contains(b));
                        values.extend(free);
                    }
                    if values.len() == s.bindings.len()
                        && values.iter().all(|v| *v < BUTTON_NAMES.len())
                        && (0..values.len()).all(|i| !values[..i].contains(&values[i]))
                    {
                        s.bindings.copy_from_slice(&values);
                    }
                }
                _ => {}
            }
        }
        if !separate_look {
            s.look_deadzone_percent = s.deadzone_percent;
        }
        s
    }
    pub fn load() -> Self {
        config_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map_or_else(Self::default, |s| Self::decode(&s))
    }
    pub fn save(&self) -> io::Result<()> {
        let p = config_path().ok_or_else(|| io::Error::other("No configuration directory"))?;
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(p, self.encode())
    }
}
pub fn config_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("Rust3D/stride-options.cfg"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
            .map(|p| p.join("Rust3D/stride-options.cfg"))
    }
}
