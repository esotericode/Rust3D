use gilrs::{Axis, Button, GamepadId, Gilrs, GilrsBuilder};
use glam::Vec2;

/// Circular deadzone with continuous rescaling; diagonal input never exceeds 1.
pub fn radial_deadzone(raw: Vec2, deadzone: f32) -> Vec2 {
    if !raw.is_finite() {
        return Vec2::ZERO;
    }
    let length = raw.length();
    let deadzone = deadzone.clamp(0., 0.95);
    if length <= deadzone {
        Vec2::ZERO
    } else {
        raw / length * ((length.min(1.) - deadzone) / (1. - deadzone))
    }
}

#[derive(Clone, Copy, Default)]
pub struct PadState {
    pub movement: Vec2,
    pub camera: Vec2,
    pub menu_axis: Vec2,
    pub jump_held: bool,
    pub jump: bool,
    pub sprint: bool,
    pub respawn: bool,
    pub recenter: bool,
    pub pause: bool,
    pub help: bool,
    pub back: bool,
}

pub struct Controller {
    gilrs: Option<Gilrs>,
    active: Option<GamepadId>,
    previous: [bool; 7],
    pub name: Option<String>,
    pub error: Option<String>,
    pub state: PadState,
}
impl Default for Controller {
    fn default() -> Self {
        // Disable the device-derived deadzone so the user-selected radial value
        // applies once to both sticks. Mappings and automatic state updates remain.
        let result = GilrsBuilder::new()
            .with_default_filters(false)
            .with_force_feedback(false)
            .build();
        let (gilrs, error) = match result {
            Ok(g) => (Some(g), None),
            Err(e) => (None, Some(e.to_string())),
        };
        Self {
            gilrs,
            active: None,
            previous: [false; 7],
            name: None,
            error,
            state: PadState::default(),
        }
    }
}
impl Controller {
    pub fn poll(&mut self, deadzone: f32) {
        self.state = PadState::default();
        let Some(g) = self.gilrs.as_mut() else {
            return;
        };
        while let Some(event) = g.next_event() {
            if matches!(event.event, gilrs::EventType::ButtonPressed(..)) {
                self.active = Some(event.id);
            }
        }
        g.inc();
        if self.active.is_none_or(|id| !g.gamepad(id).is_connected()) {
            self.active = g
                .gamepads()
                .find(|(_, p)| p.is_connected())
                .map(|(id, _)| id);
            self.previous = [false; 7];
        }
        let Some(id) = self.active else {
            self.name = None;
            self.previous = [false; 7];
            return;
        };
        let p = g.gamepad(id);
        self.name = Some(p.name().to_owned());
        let movement = radial_deadzone(
            Vec2::new(p.value(Axis::LeftStickX), p.value(Axis::LeftStickY)),
            deadzone,
        );
        let camera = radial_deadzone(
            Vec2::new(p.value(Axis::RightStickX), p.value(Axis::RightStickY)),
            deadzone,
        );
        let dpad = Vec2::new(
            p.value(Axis::DPadX)
                + (p.is_pressed(Button::DPadRight) as i32 - p.is_pressed(Button::DPadLeft) as i32)
                    as f32,
            p.value(Axis::DPadY)
                + (p.is_pressed(Button::DPadUp) as i32 - p.is_pressed(Button::DPadDown) as i32)
                    as f32,
        )
        .clamp_length_max(1.);
        let buttons = [
            p.is_pressed(Button::South),
            p.is_pressed(Button::West),
            p.is_pressed(Button::North),
            p.is_pressed(Button::Start),
            p.is_pressed(Button::Select),
            p.is_pressed(Button::East),
            p.is_pressed(Button::RightTrigger2),
        ];
        let pressed = std::array::from_fn::<_, 7, _>(|i| buttons[i] && !self.previous[i]);
        self.previous = buttons;
        self.state = PadState {
            movement,
            camera,
            menu_axis: if dpad.length_squared() > 0. {
                dpad
            } else {
                movement
            },
            jump_held: buttons[0],
            jump: pressed[0],
            sprint: buttons[5] || buttons[6] || p.is_pressed(Button::RightTrigger),
            respawn: pressed[1],
            recenter: pressed[2],
            pause: pressed[3],
            help: pressed[4],
            back: pressed[5],
        };
    }
}
