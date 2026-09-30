use stride::{
    engine::ui::{Ui, INK, MUTED, WHITE},
    settings::{Settings, BINDING_NAMES, BUTTON_NAMES, FRAME_CAPS, RESOLUTIONS},
    world::{MINT, NAVY},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Screen {
    Pause,
    Options,
    Camera,
    Controller,
    Audio,
    Help,
    ConfirmRestart,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    None,
    Resume,
    Options,
    Camera,
    Controller,
    Audio,
    Help,
    Respawn,
    RequestRestart,
    Restart,
    Quit,
    Apply,
    Back,
}
pub struct Menu {
    pub screen: Option<Screen>,
    pub selected: usize,
    pub draft: Settings,
    pub status: String,
}
impl Menu {
    pub fn new(settings: Settings) -> Self {
        Self {
            screen: None,
            selected: 0,
            draft: settings,
            status: String::new(),
        }
    }
    pub fn open(&mut self, screen: Screen, settings: &Settings) {
        self.draft = settings.clone();
        self.open_sub(screen);
    }
    pub fn open_sub(&mut self, screen: Screen) {
        self.screen = Some(screen);
        self.selected = 0;
        self.status.clear();
    }
    pub fn rows(&self) -> usize {
        match self.screen {
            Some(Screen::Pause) => 6,
            Some(Screen::Options) => 8,
            Some(Screen::Camera) => 5,
            Some(Screen::Controller) => 9,
            Some(Screen::Audio) => 3,
            Some(Screen::Help) => 1,
            Some(Screen::ConfirmRestart) => 2,
            None => 0,
        }
    }
    pub fn navigate(&mut self, direction: i32) {
        let count = self.rows();
        if count > 0 {
            self.selected = (self.selected as i32 + direction).rem_euclid(count as i32) as usize;
        }
    }
    pub fn adjust(&mut self, direction: i32) {
        match self.screen {
            Some(Screen::Options) => match self.selected {
                0 => {
                    self.draft.resolution = (self.draft.resolution as i32 + direction)
                        .rem_euclid(RESOLUTIONS.len() as i32)
                        as usize
                }
                1 => {
                    self.draft.frame_cap = (self.draft.frame_cap as i32 + direction)
                        .rem_euclid(FRAME_CAPS.len() as i32)
                        as usize
                }
                2 => self.draft.fullscreen = !self.draft.fullscreen,
                _ => {}
            },
            Some(Screen::Camera) => match self.selected {
                0 => {
                    self.draft.camera_sensitivity = ((self.draft.camera_sensitivity as i32 - 25
                        + direction * 25)
                        .rem_euclid(200)
                        + 25) as u32
                }
                1 => self.draft.invert_y = !self.draft.invert_y,
                2 => self.draft.auto_camera = !self.draft.auto_camera,
                _ => {}
            },
            Some(Screen::Controller) => match self.selected {
                0 => {
                    self.draft.deadzone_percent =
                        (self.draft.deadzone_percent as i32 + direction * 5).rem_euclid(35) as u32
                }
                1 => {
                    self.draft.look_deadzone_percent =
                        (self.draft.look_deadzone_percent as i32 + direction * 5).rem_euclid(35)
                            as u32
                }
                2..=5 => {
                    let action = self.selected - 2;
                    let button =
                        (self.draft.bindings[action] as i32 + direction).rem_euclid(8) as usize;
                    self.draft.bind(action, button);
                }
                6 => self.draft.vibration = !self.draft.vibration,
                _ => {}
            },
            Some(Screen::Audio) if self.selected == 0 => {
                self.draft.volume =
                    (self.draft.volume as i32 + direction * 10).rem_euclid(110) as u32
            }
            _ => {}
        }
    }
    pub fn confirm(&mut self) -> Action {
        match self.screen {
            Some(Screen::Pause) => [
                Action::Resume,
                Action::Options,
                Action::Help,
                Action::Respawn,
                Action::RequestRestart,
                Action::Quit,
            ][self.selected],
            Some(Screen::Options) => match self.selected {
                3 => Action::Camera,
                4 => Action::Controller,
                5 => Action::Audio,
                6 => Action::Apply,
                7 => Action::Back,
                _ => {
                    self.adjust(1);
                    Action::None
                }
            },
            Some(Screen::Camera) => match self.selected {
                3 => Action::Apply,
                4 => Action::Back,
                _ => {
                    self.adjust(1);
                    Action::None
                }
            },
            Some(Screen::Controller) => match self.selected {
                7 => Action::Apply,
                8 => Action::Back,
                _ => {
                    self.adjust(1);
                    Action::None
                }
            },
            Some(Screen::Audio) => match self.selected {
                1 => Action::Apply,
                2 => Action::Back,
                _ => {
                    self.adjust(1);
                    Action::None
                }
            },
            Some(Screen::ConfirmRestart) => {
                if self.selected == 0 {
                    Action::Back
                } else {
                    Action::Restart
                }
            }
            Some(Screen::Help) => Action::Back,
            None => Action::None,
        }
    }
    fn top(&self) -> f32 {
        if self.screen == Some(Screen::Help) {
            566.
        } else {
            190.
        }
    }
    pub fn row_at(&self, x: f32, y: f32) -> Option<usize> {
        let top = self.top();
        if !(340. ..=940.).contains(&x) || y < top {
            return None;
        }
        let row = ((y - top) / 36.) as usize;
        (row < self.rows() && (y - top) % 36. <= 30.).then_some(row)
    }
    pub fn draw(
        &self,
        ui: &mut Ui,
        applied: &Settings,
        pad: Option<&str>,
        fps: f32,
        window: (f32, f32),
    ) {
        let Some(screen) = self.screen else {
            return;
        };
        ui.rect(310., 90., 660., 552., INK);
        ui.rect(310., 90., 660., 4., MINT);
        ui.text(
            342.,
            119.,
            match screen {
                Screen::Pause => "TAKE A BREATHER",
                Screen::Options => "OPTIONS",
                Screen::Camera => "CAMERA",
                Screen::Controller => "CONTROLLER",
                Screen::Audio => "SOUND",
                Screen::Help => "LEARN THE MOVES",
                Screen::ConfirmRestart => "RESTART THE COURSE?",
            },
            3.,
            WHITE,
        );
        if screen == Screen::Help {
            let labels = [
                ("WASD / LEFT STICK".into(), "MOVE / LIGHT STICK INPUT WALKS"),
                (
                    format!("SPACE / {}", BUTTON_NAMES[applied.bindings[0]]),
                    "TAP HOP / HOLD HIGH JUMP",
                ),
                (
                    format!("SHIFT / {}", BUTTON_NAMES[applied.bindings[2]]),
                    "SPRINT / ADD JUMP FOR LONG JUMP",
                ),
                (
                    format!("F / {}", BUTTON_NAMES[applied.bindings[1]]),
                    "DIVE / JUMP ON LANDING TO ROLL OUT",
                ),
                ("SPACE / JUMP".into(), "PRESS AT WALL CONTACT TO KICK"),
                ("Q-E / RIGHT STICK".into(), "CAMERA / RIGHT MOUSE DRAG ALSO"),
                (
                    format!("C / {}", BUTTON_NAMES[applied.bindings[3]]),
                    "RECENTER CAMERA",
                ),
                ("ESC / START".into(), "PAUSE / RESPAWN / RESTART / OPTIONS"),
            ];
            for (i, (key, meaning)) in labels.iter().enumerate() {
                let y = 184. + i as f32 * 36.;
                ui.text(342., y, key, 1.2, MINT);
                ui.text(579., y, meaning, 1.15, WHITE);
            }
            ui.text(
                342.,
                495.,
                "DIVE ONCE PER FLIGHT. PRESS JUMP DURING THE LANDING SLIDE.",
                1.2,
                MUTED,
            );
            ui.text(
                342.,
                520.,
                "RESPAWN AND RESTART ARE MENU ACTIONS. RESTART NEEDS CONFIRMATION.",
                1.15,
                MUTED,
            );
            self.row(ui, 0, 566., "BACK", "");
            return;
        }
        ui.text(
            342.,
            167.,
            if screen == Screen::ConfirmRestart {
                "THIS CLEARS YOUR BEACONS AND CURRENT RUN TIMER."
            } else {
                "UP/DOWN SELECT   LEFT/RIGHT CHANGE   ENTER OR A CONFIRM"
            },
            1.3,
            MUTED,
        );
        let on = |b| if b { "ON".into() } else { "OFF".into() };
        let values: Vec<String> = match screen {
            Screen::Options => {
                let (w, h) = self.draft.size();
                vec![
                    format!("{w} X {h}"),
                    self.draft
                        .cap()
                        .map_or("UNCAPPED".into(), |c| format!("{c} FPS")),
                    if self.draft.fullscreen {
                        "FULLSCREEN".into()
                    } else {
                        "WINDOWED".into()
                    },
                    "".into(),
                    "".into(),
                    "".into(),
                    "".into(),
                    "".into(),
                ]
            }
            Screen::Camera => vec![
                format!("{} PERCENT", self.draft.camera_sensitivity),
                on(self.draft.invert_y),
                on(self.draft.auto_camera),
                "".into(),
                "".into(),
            ],
            Screen::Controller => vec![
                format!("{} PERCENT", self.draft.deadzone_percent),
                format!("{} PERCENT", self.draft.look_deadzone_percent),
                BUTTON_NAMES[self.draft.bindings[0]].into(),
                BUTTON_NAMES[self.draft.bindings[1]].into(),
                BUTTON_NAMES[self.draft.bindings[2]].into(),
                BUTTON_NAMES[self.draft.bindings[3]].into(),
                on(self.draft.vibration),
                "".into(),
                "".into(),
            ],
            Screen::Audio => vec![
                format!("{} PERCENT", self.draft.volume),
                "".into(),
                "".into(),
            ],
            _ => vec![String::new(); self.rows()],
        };
        let labels: &[&str] = match screen {
            Screen::Pause => &[
                "RESUME",
                "OPTIONS",
                "CONTROLS",
                "RESPAWN AT CHECKPOINT",
                "RESTART COURSE",
                "EXIT GAME",
            ],
            Screen::Options => &[
                "RENDER RESOLUTION",
                "FRAME-RATE CAP",
                "DISPLAY MODE",
                "CAMERA SETTINGS",
                "CONTROLLER SETTINGS",
                "SOUND SETTINGS",
                "APPLY AND SAVE",
                "BACK",
            ],
            Screen::Camera => &[
                "LOOK SENSITIVITY",
                "INVERT VERTICAL LOOK",
                "AUTO ALIGN BEHIND PLAYER",
                "APPLY AND SAVE",
                "BACK",
            ],
            Screen::Controller => &[
                "MOVEMENT DEADZONE",
                "CAMERA DEADZONE",
                BINDING_NAMES[0],
                BINDING_NAMES[1],
                BINDING_NAMES[2],
                BINDING_NAMES[3],
                "VIBRATION",
                "APPLY AND SAVE",
                "BACK",
            ],
            Screen::Audio => &["EFFECTS VOLUME", "APPLY AND SAVE", "BACK"],
            Screen::ConfirmRestart => &["CANCEL / KEEP PLAYING", "RESTART FROM THE BEGINNING"],
            Screen::Help => unreachable!(),
        };
        for (i, label) in labels.iter().enumerate() {
            self.row(ui, i, 190. + i as f32 * 36., label, &values[i]);
        }
        match screen {
            Screen::Options => {
                let (w, h) = applied.size();
                ui.text(
                    342.,
                    503.,
                    &format!(
                        "RENDER {w} X {h} / WINDOW {:.0} X {:.0}",
                        window.0, window.1
                    ),
                    1.3,
                    MUTED,
                );
                ui.text(
                    342.,
                    527.,
                    &format!("ACTUAL {:.0} FPS / MOVEMENT SIMULATION 120 HZ", fps),
                    1.3,
                    MINT,
                );
            }
            Screen::Controller => {
                ui.text(
                    342.,
                    531.,
                    "DUPLICATE BINDINGS SWAP. MENU A/B AND START STAY FIXED.",
                    1.2,
                    MUTED,
                );
                ui.text(
                    342.,
                    551.,
                    if pad.is_some() {
                        "CONTROLLER CONNECTED"
                    } else {
                        "NO CONTROLLER CONNECTED"
                    },
                    1.2,
                    MINT,
                );
            }
            Screen::Camera => ui.text(
                342.,
                503.,
                "AUTO ALIGN PAUSES WHILE YOU CONTROL THE CAMERA.",
                1.3,
                MUTED,
            ),
            Screen::Audio => ui.text(
                342.,
                503.,
                "PROCEDURAL FOOTSTEPS / JUMP / LAND / WALL KICK / DIVE.",
                1.2,
                MUTED,
            ),
            Screen::Pause => ui.text(
                342.,
                503.,
                "ESC OR START TO RESUME. RESET ACTIONS LIVE HERE.",
                1.3,
                MUTED,
            ),
            _ => {}
        }
        ui.text(342., 579., "ESC OR B BACK", 1.3, WHITE);
        ui.text(342., 607., &self.status, 1.3, MINT);
    }
    fn row(&self, ui: &mut Ui, index: usize, y: f32, label: &str, value: &str) {
        let selected = self.selected == index;
        ui.rect(340., y, 600., 30., if selected { NAVY } else { INK });
        if selected {
            ui.rect(340., y, 3., 30., MINT);
        }
        ui.text(
            354.,
            y + 10.,
            label,
            1.5,
            if selected { WHITE } else { MUTED },
        );
        ui.text(712., y + 10., value, 1.5, MINT);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn options_keep_drafts_between_pages_and_apply_explicitly() {
        let original = Settings::default();
        let mut menu = Menu::new(original.clone());
        menu.open(Screen::Options, &original);
        menu.adjust(1);
        assert_eq!(original.size(), (1280, 720));
        assert_eq!(menu.draft.size(), (1600, 900));
        menu.open_sub(Screen::Camera);
        menu.adjust(1);
        menu.open_sub(Screen::Options);
        assert_eq!(menu.draft.size(), (1600, 900));
        menu.selected = 6;
        assert_eq!(menu.confirm(), Action::Apply);
        menu.open(Screen::Options, &original);
        assert_eq!(menu.draft, original);
    }
    #[test]
    fn restart_requires_explicit_confirmation_and_default_is_cancel() {
        let settings = Settings::default();
        let mut menu = Menu::new(settings.clone());
        menu.open(Screen::Pause, &settings);
        menu.selected = 4;
        assert_eq!(menu.confirm(), Action::RequestRestart);
        menu.open_sub(Screen::ConfirmRestart);
        assert_eq!(menu.confirm(), Action::Back);
        menu.navigate(1);
        assert_eq!(menu.confirm(), Action::Restart);
        menu.open(Screen::Pause, &settings);
        menu.selected = 3;
        assert_eq!(menu.confirm(), Action::Respawn);
    }
    #[test]
    fn every_visible_row_is_clickable_and_wraps() {
        let settings = Settings::default();
        let mut menu = Menu::new(settings.clone());
        for screen in [
            Screen::Pause,
            Screen::Options,
            Screen::Camera,
            Screen::Controller,
            Screen::Audio,
            Screen::Help,
            Screen::ConfirmRestart,
        ] {
            menu.open(screen, &settings);
            for i in 0..menu.rows() {
                assert_eq!(
                    menu.row_at(760., menu.top() + i as f32 * 36. + 15.),
                    Some(i)
                );
            }
            assert_eq!(
                menu.row_at(760., menu.top() + menu.rows() as f32 * 36.),
                None
            );
            menu.navigate(-1);
            assert_eq!(menu.selected, menu.rows() - 1);
        }
    }
}
