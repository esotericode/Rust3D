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

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub resolution: usize,
    pub frame_cap: usize,
    pub fullscreen: bool,
    pub deadzone_percent: u32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            resolution: 1,
            frame_cap: 1,
            fullscreen: false,
            deadzone_percent: 10,
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
    pub fn encode(&self) -> String {
        format!(
            "resolution={}\nframe_cap={}\nfullscreen={}\ndeadzone={}\n",
            self.resolution, self.frame_cap, self.fullscreen, self.deadzone_percent
        )
    }
    pub fn decode(text: &str) -> Self {
        let mut s = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key {
                "resolution" => {
                    if let Ok(v) = value.parse::<usize>() {
                        if v < RESOLUTIONS.len() {
                            s.resolution = v;
                        }
                    }
                }
                "frame_cap" => {
                    if let Ok(v) = value.parse::<usize>() {
                        if v < FRAME_CAPS.len() {
                            s.frame_cap = v;
                        }
                    }
                }
                "fullscreen" => {
                    if let Ok(v) = value.parse() {
                        s.fullscreen = v;
                    }
                }
                "deadzone" => {
                    if let Ok(v) = value.parse::<u32>() {
                        if v <= 30 && v % 5 == 0 {
                            s.deadzone_percent = v;
                        }
                    }
                }
                _ => {}
            }
        }
        s
    }
    pub fn load() -> Self {
        path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map_or_else(Self::default, |s| Self::decode(&s))
    }
    pub fn save(&self) -> io::Result<()> {
        let p = path().ok_or_else(|| io::Error::other("No configuration directory"))?;
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(p, self.encode())
    }
}
fn path() -> Option<PathBuf> {
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
