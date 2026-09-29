use stride::{
    engine::ui::{Ui, INK, MUTED, WHITE},
    settings::{Settings, FRAME_CAPS, RESOLUTIONS},
    world::{MINT, NAVY},
};

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Pause,
    Options,
    Help,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn options_edits_are_drafts_until_apply() {
        let original = Settings::default();
        let mut menu = Menu::new(original.clone());
        menu.open(Screen::Options, &original);
        menu.adjust(1);
        assert_eq!(original.size(), (1280, 720));
        assert_eq!(menu.draft.size(), (1600, 900));
        menu.navigate(4);
        assert!(matches!(menu.confirm(), Action::Apply));
        menu.open(Screen::Options, &original);
        assert_eq!(menu.draft, original);
    }
    #[test]
    fn menu_navigation_and_mouse_rows_match_visible_buttons() {
        let settings = Settings::default();
        let mut menu = Menu::new(settings.clone());
        menu.open(Screen::Pause, &settings);
        menu.navigate(-1);
        assert!(matches!(menu.confirm(), Action::Quit));
        menu.navigate(2);
        assert!(matches!(menu.confirm(), Action::Options));
        menu.open(Screen::Options, &settings);
        assert_eq!(menu.row_at(760., 392.), Some(4));
        assert_eq!(menu.row_at(760., 475.), None);
        menu.selected = 3;
        menu.adjust(-1);
        assert_eq!(menu.draft.deadzone_percent, 5);
        menu.adjust(-1);
        assert_eq!(menu.draft.deadzone_percent, 0);
        menu.adjust(-1);
        assert_eq!(menu.draft.deadzone_percent, 30);
    }
}
#[derive(Clone, Copy)]
pub enum Action {
    None,
    Resume,
    Options,
    Help,
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
        self.screen = Some(screen);
        self.selected = 0;
        self.draft = settings.clone();
        self.status.clear();
    }
    pub fn rows(&self) -> usize {
        match self.screen {
            Some(Screen::Pause) => 5,
            Some(Screen::Options) => 6,
            Some(Screen::Help) => 1,
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
        if self.screen != Some(Screen::Options) {
            return;
        }
        match self.selected {
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
            3 => {
                self.draft.deadzone_percent =
                    (self.draft.deadzone_percent as i32 + direction * 5).rem_euclid(35) as u32
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
                Action::Restart,
                Action::Quit,
            ][self.selected],
            Some(Screen::Options) => match self.selected {
                4 => Action::Apply,
                5 => Action::Back,
                _ => {
                    self.adjust(1);
                    Action::None
                }
            },
            Some(Screen::Help) => Action::Back,
            None => Action::None,
        }
    }
    pub fn row_at(&self, x: f32, y: f32) -> Option<usize> {
        let top = match self.screen {
            Some(Screen::Pause) => 230.,
            Some(Screen::Options) => 188.,
            Some(Screen::Help) => 566.,
            None => return None,
        };
        let spacing = if self.screen == Some(Screen::Help) {
            42.
        } else {
            48.
        };
        if !(340. ..=940.).contains(&x) || y < top {
            return None;
        }
        let row = ((y - top) / spacing) as usize;
        (row < self.rows() && (y - top) % spacing <= 38.).then_some(row)
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
                Screen::Help => "LEARN THE MOVES",
            },
            3.,
            WHITE,
        );
        match screen {
            Screen::Pause => {
                ui.text(
                    342.,
                    171.,
                    "UP/DOWN SELECT   ENTER OR A CONFIRM",
                    1.5,
                    MUTED,
                );
                for (i, label) in [
                    "RESUME",
                    "OPTIONS",
                    "CONTROLS",
                    "RESTART COURSE",
                    "EXIT GAME",
                ]
                .into_iter()
                .enumerate()
                {
                    self.row(ui, i, 230. + i as f32 * 48., label, "");
                }
                ui.text(
                    342.,
                    521.,
                    if pad.is_some() {
                        "CONTROLLER CONNECTED"
                    } else {
                        "KEYBOARD / MOUSE"
                    },
                    1.5,
                    MINT,
                );
                ui.text(342., 554., "ESC OR START TO RESUME", 1.5, MUTED);
            }
            Screen::Options => {
                let (w, h) = self.draft.size();
                let values = [
                    format!("{w} X {h}"),
                    self.draft
                        .cap()
                        .map_or("UNCAPPED".into(), |c| format!("{c} FPS")),
                    if self.draft.fullscreen {
                        "FULLSCREEN".into()
                    } else {
                        "WINDOWED".into()
                    },
                    format!("{} PERCENT", self.draft.deadzone_percent),
                    String::new(),
                    String::new(),
                ];
                for (i, label) in [
                    "RENDER RESOLUTION",
                    "FRAME-RATE CAP",
                    "DISPLAY MODE",
                    "STICK DEADZONE",
                    "APPLY AND SAVE",
                    "BACK",
                ]
                .into_iter()
                .enumerate()
                {
                    self.row(ui, i, 188. + i as f32 * 48., label, &values[i]);
                }
                let (w, h) = applied.size();
                ui.text(
                    342.,
                    494.,
                    &format!(
                        "RENDER {w} X {h} / WINDOW {:.0} X {:.0}",
                        window.0, window.1
                    ),
                    1.4,
                    MUTED,
                );
                ui.text(
                    342.,
                    517.,
                    &format!("ACTUAL {:.0} FPS / MOVEMENT SIMULATION 120 HZ", fps),
                    1.4,
                    MINT,
                );
                ui.text(
                    342.,
                    540.,
                    "FULLSCREEN SCALES TO YOUR DISPLAY. ASPECT IS PRESERVED.",
                    1.2,
                    MUTED,
                );
                ui.text(
                    342.,
                    562.,
                    "LEFT/RIGHT CHANGE   ENTER/A APPLY   ESC/B BACK",
                    1.3,
                    WHITE,
                );
                ui.text(342., 598., &self.status, 1.4, MINT);
            }
            Screen::Help => {
                for (i, (key, meaning)) in [
                    ("WASD / LEFT STICK", "MOVE / LIGHT STICK INPUT WALKS"),
                    ("SPACE / A-CROSS", "TAP FOR HOP / HOLD FOR HIGH JUMP"),
                    ("SHIFT / RT OR B", "SPRINT / ADD JUMP FOR LONG JUMP"),
                    ("SPACE / A-CROSS", "PRESS AT WALL CONTACT TO KICK"),
                    ("Q-E / RIGHT STICK", "CAMERA / RIGHT MOUSE DRAG ALSO"),
                    ("R / X-SQUARE", "RESPAWN AT YOUR CHECKPOINT"),
                    ("Y-TRIANGLE", "RECENTER CAMERA"),
                    ("ESC / START", "PAUSE / OPEN OPTIONS"),
                ]
                .into_iter()
                .enumerate()
                {
                    let y = 184. + i as f32 * 36.;
                    ui.text(342., y, key, 1.4, MINT);
                    ui.text(568., y, meaning, 1.3, WHITE);
                }
                ui.text(
                    342.,
                    496.,
                    "ALTERNATE WALLS. THE WALL CONTACT WINDOW IS SHORT.",
                    1.4,
                    MUTED,
                );
                ui.text(
                    342.,
                    523.,
                    "F1 OR SELECT: HELP   F2: OPTIONS   F11: FULLSCREEN",
                    1.3,
                    MUTED,
                );
                self.row(ui, 0, 566., "BACK", "");
            }
        }
    }
    fn row(&self, ui: &mut Ui, index: usize, y: f32, label: &str, value: &str) {
        let selected = self.selected == index;
        ui.rect(340., y, 600., 38., if selected { NAVY } else { INK });
        if selected {
            ui.rect(340., y, 3., 38., MINT);
        }
        ui.text(
            354.,
            y + 12.,
            label,
            1.8,
            if selected { WHITE } else { MUTED },
        );
        ui.text(723., y + 12., value, 1.8, MINT);
    }
}
