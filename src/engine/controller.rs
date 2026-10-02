use crate::settings::Settings;
use gilrs::{
    ff::{BaseEffect, BaseEffectType, Effect, EffectBuilder, Repeat, Ticks},
    Axis, Button, GamepadId, Gilrs, GilrsBuilder,
};
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
    pub dive: bool,
    pub sprint: bool,
    pub crouch: bool,
    pub recenter: bool,
    pub pause: bool,
    pub help: bool,
    pub back: bool,
    pub confirm: bool,
}
/// Menus always use A/B; gameplay bindings never include a reset action.
pub fn map_buttons(buttons: [bool; 10], previous: [bool; 10], settings: &Settings) -> PadState {
    let edge = |i: usize| buttons[i] && !previous[i];
    PadState {
        jump_held: buttons[settings.bindings[0]],
        jump: edge(settings.bindings[0]),
        dive: edge(settings.bindings[1]),
        sprint: buttons[settings.bindings[2]],
        recenter: edge(settings.bindings[3]),
        crouch: buttons[settings.bindings[4]],
        pause: edge(8),
        help: edge(9),
        confirm: edge(0),
        back: edge(1),
        ..Default::default()
    }
}
pub struct Controller {
    gilrs: Option<Gilrs>,
    active: Option<GamepadId>,
    previous: [bool; 10],
    effect: Option<Effect>,
    pub name: Option<String>,
    pub error: Option<String>,
    pub state: PadState,
}
impl Default for Controller {
    fn default() -> Self {
        let make = |ff| {
            GilrsBuilder::new()
                .with_default_filters(false)
                .with_force_feedback(ff)
                .build()
                .map_err(|e| e.to_string())
        };
        let result = make(true).or_else(|_| make(false));
        let (gilrs, error) = match result {
            Ok(g) => (Some(g), None),
            Err(e) => (None, Some(e.to_string())),
        };
        Self {
            gilrs,
            active: None,
            previous: [false; 10],
            effect: None,
            name: None,
            error,
            state: PadState::default(),
        }
    }
}
impl Controller {
    pub fn poll(&mut self, settings: &Settings) {
        self.state = PadState::default();
        let Some(g) = self.gilrs.as_mut() else {
            return;
        };
        while let Some(event) = g.next_event() {
            if matches!(event.event, gilrs::EventType::ButtonPressed(..)) {
                if self.active != Some(event.id) {
                    self.previous = [false; 10];
                }
                self.active = Some(event.id);
            }
        }
        g.inc();
        if self.active.is_none_or(|id| !g.gamepad(id).is_connected()) {
            self.active = g
                .gamepads()
                .find(|(_, p)| p.is_connected())
                .map(|(id, _)| id);
            self.previous = [false; 10];
        }
        let Some(id) = self.active else {
            self.name = None;
            return;
        };
        let p = g.gamepad(id);
        self.name = Some(p.name().to_owned());
        let movement = radial_deadzone(
            Vec2::new(p.value(Axis::LeftStickX), p.value(Axis::LeftStickY)),
            settings.deadzone(),
        );
        let camera = radial_deadzone(
            Vec2::new(p.value(Axis::RightStickX), p.value(Axis::RightStickY)),
            settings.look_deadzone(),
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
            Button::South,
            Button::East,
            Button::West,
            Button::North,
            Button::LeftTrigger,
            Button::RightTrigger,
            Button::LeftTrigger2,
            Button::RightTrigger2,
            Button::Start,
            Button::Select,
        ]
        .map(|b| p.is_pressed(b));
        self.state = map_buttons(buttons, self.previous, settings);
        self.previous = buttons;
        self.state.movement = movement;
        self.state.camera = camera;
        self.state.menu_axis = if dpad.length_squared() > 0. {
            dpad
        } else {
            movement
        };
    }
    pub fn rumble(&mut self, enabled: bool, strong: u16, weak: u16) {
        if !enabled {
            return;
        }
        let (Some(g), Some(id)) = (self.gilrs.as_mut(), self.active) else {
            return;
        };
        if !g.gamepad(id).is_connected() || !g.gamepad(id).is_ff_supported() {
            return;
        }
        self.effect = EffectBuilder::new()
            .add_effect(BaseEffect {
                kind: BaseEffectType::Strong { magnitude: strong },
                ..Default::default()
            })
            .add_effect(BaseEffect {
                kind: BaseEffectType::Weak { magnitude: weak },
                ..Default::default()
            })
            .repeat(Repeat::For(Ticks::from_ms(80)))
            .gamepads(&[id])
            .finish(g)
            .ok();
        if let Some(effect) = &self.effect {
            let _ = effect.play();
        }
    }
    pub fn stop_rumble(&mut self) {
        if let Some(effect) = self.effect.take() {
            let _ = effect.stop();
        }
    }
}
