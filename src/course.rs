//! Deterministic kinematic platforms and the ordered Skyway challenge.
use crate::{
    engine::physics::Player,
    world::{Solid, World, NAVY},
};
use glam::{vec3, Vec3};

pub const AMBER: [f32; 3] = [1., 0.70, 0.25];
pub const VIOLET: [f32; 3] = [0.76, 0.63, 1.];
pub const SECTIONS: [(&str, &str); 8] = [
    ("BOARDING", "BOARD THE SHUTTLE. RIDE TO THE DOCK."),
    ("CROSSWIND", "WAIT FOR ALIGNMENT. AIM FOR THE CENTRE."),
    ("LIFT WORKS", "RIDE UP. JUMP AT THE TOP OF THE LIFT."),
    ("DIAGONAL DOCK", "RIDE FIRST. JUMP WHEN THE DOCK IS CLOSE."),
    ("CLOCKWORK", "SHORT HOPS. WATCH THE NEXT PLATFORM."),
    ("LEAP AND DIVE", "LONG JUMP. DIVE UNDER THE ARCH. ROLL OUT."),
    ("HIGH HARBOR", "DIAGONAL RIDES. KEEP YOUR LANDINGS CALM."),
    ("SUMMIT", "LIFT, NARROW DECKS, THEN THE FINAL SHUTTLE."),
];

#[derive(Clone, Debug)]
pub struct MovingPlatform {
    pub origin: Vec3,
    pub size: Vec3,
    pub travel: Vec3,
    pub period: f64,
    pub phase: f64,
    pub dwell: f64,
    pub solid: Solid,
    pub previous_center: Vec3,
    pub delta: Vec3,
    pub velocity: Vec3,
}
impl MovingPlatform {
    pub fn new(top: Vec3, size: Vec3, travel: Vec3, period: f64, phase: f64) -> Self {
        assert!(period > 2.4);
        let origin = top - Vec3::Y * size.y * 0.5;
        let mut p = Self {
            origin,
            size,
            travel,
            period,
            phase,
            dwell: 0.9,
            solid: Solid::new(origin, size, AMBER),
            previous_center: origin,
            delta: Vec3::ZERO,
            velocity: Vec3::ZERO,
        };
        p.reset(0.);
        p
    }
    pub fn center_at(&self, time: f64) -> Vec3 {
        let t = (time + self.phase).rem_euclid(self.period);
        let leg = (self.period - 2. * self.dwell) * 0.5;
        let u = if t < self.dwell {
            0.
        } else if t < self.dwell + leg {
            (t - self.dwell) / leg
        } else if t < 2. * self.dwell + leg {
            1.
        } else {
            1. - (t - 2. * self.dwell - leg) / leg
        };
        let u = u as f32;
        self.origin + self.travel * (u * u * (3. - 2. * u))
    }
    pub fn reset(&mut self, time: f64) {
        let center = self.center_at(time);
        self.solid = Solid::new(center, self.size, AMBER);
        self.previous_center = center;
        self.delta = Vec3::ZERO;
        self.velocity = Vec3::ZERO;
    }
    pub fn advance(&mut self, time: f64, dt: f32) {
        self.previous_center = (self.solid.min + self.solid.max) * 0.5;
        let center = self.center_at(time);
        self.delta = center - self.previous_center;
        self.velocity = self.delta / dt;
        self.solid = Solid::new(center, self.size, AMBER);
    }
    pub fn rendered_center(&self, alpha: f32) -> Vec3 {
        self.previous_center
            .lerp((self.solid.min + self.solid.max) * 0.5, alpha)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Surface {
    Fixed(usize),
    Moving(usize),
}
#[derive(Clone, Debug)]
pub struct Node {
    pub surface: Surface,
    pub section: usize,
    pub checkpoint: bool,
    pub offset: Vec3,
}
impl Node {
    pub fn solid<'a>(&self, world: &'a World) -> &'a Solid {
        match self.surface {
            Surface::Fixed(i) => &world.solids[i],
            Surface::Moving(i) => &world.platforms[i].solid,
        }
    }
    pub fn position(&self, world: &World) -> Vec3 {
        let s = self.solid(world);
        vec3(
            (s.min.x + s.max.x) * 0.5,
            s.max.y,
            (s.min.z + s.max.z) * 0.5,
        ) + self.offset
    }
    pub fn reached(&self, world: &World, player: &Player) -> bool {
        let s = self.solid(world);
        let p = player.pos;
        player.grounded
            && (p.y - s.max.y).abs() < 0.12
            && p.x >= s.min.x + 0.25
            && p.x <= s.max.x - 0.25
            && p.z >= s.min.z + 0.25
            && p.z <= s.max.z - 0.25
            && (self.offset == Vec3::ZERO || (p - self.position(world)).length() < 1.7)
            && match self.surface {
                Surface::Moving(i) => player.ground_platform == Some(i),
                _ => true,
            }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Course {
    pub nodes: Vec<Node>,
    pub checkpoints: Vec<usize>,
}
impl Course {
    pub fn build(world: &mut World) {
        let mut c = Self::default();
        #[allow(clippy::too_many_arguments)]
        fn deck(c: &mut Course, w: &mut World, s: usize, p: Vec3, x: f32, z: f32, cp: bool) {
            let i = w.solids.len();
            w.solids
                .push(Solid::new(p - Vec3::Y * 0.35, vec3(x, 0.7, z), NAVY));
            if cp {
                w.solids.push(Solid::new(
                    vec3(p.x, (p.y - 0.7) * 0.5, p.z),
                    vec3(1.2, p.y - 0.7, 1.2),
                    NAVY,
                ));
                c.checkpoints.push(c.nodes.len());
            }
            c.nodes.push(Node {
                surface: Surface::Fixed(i),
                section: s,
                checkpoint: cp,
                offset: Vec3::ZERO,
            });
        }
        #[allow(clippy::too_many_arguments)]
        fn ferry(
            c: &mut Course,
            w: &mut World,
            s: usize,
            p: Vec3,
            x: f32,
            z: f32,
            travel: Vec3,
            period: f64,
            phase: f64,
        ) {
            let i = w.platforms.len();
            w.platforms.push(MovingPlatform::new(
                p,
                vec3(x, 0.7, z),
                travel,
                period,
                phase,
            ));
            c.nodes.push(Node {
                surface: Surface::Moving(i),
                section: s,
                checkpoint: false,
                offset: Vec3::ZERO,
            });
        }
        deck(&mut c, world, 0, vec3(56., 8., 32.), 10., 8., true);
        ferry(
            &mut c,
            world,
            0,
            vec3(56., 8., 24.),
            5.,
            5.,
            vec3(0., 0., -16.),
            14.,
            0.,
        );
        deck(&mut c, world, 0, vec3(56., 8., 0.), 8., 8., true);
        deck(&mut c, world, 1, vec3(66., 8., 0.), 6., 6., false);
        for (j, x) in [76., 86., 96.].into_iter().enumerate() {
            ferry(
                &mut c,
                world,
                1,
                vec3(x, 8. + j as f32 * 0.6, -4.),
                4.8,
                3.8,
                vec3(0., 0., 8.),
                7. + j as f64,
                j as f64 * 1.4,
            );
            deck(
                &mut c,
                world,
                1,
                vec3(x + 5., 8.2 + j as f32 * 0.6, 0.),
                4.,
                4.,
                false,
            );
        }
        deck(&mut c, world, 1, vec3(110., 10., 0.), 8., 10., true);
        ferry(
            &mut c,
            world,
            2,
            vec3(120., 10., 0.),
            5.,
            5.,
            vec3(0., 8., 0.),
            10.,
            0.,
        );
        deck(&mut c, world, 2, vec3(128., 19., 0.), 6., 6., false);
        deck(&mut c, world, 2, vec3(133., 20., 4.), 4.4, 4.4, false);
        deck(&mut c, world, 2, vec3(128., 21., 8.), 4.4, 4.4, false);
        deck(&mut c, world, 2, vec3(133., 22., 12.), 4.4, 4.4, false);
        deck(&mut c, world, 2, vec3(140., 22., 16.), 8., 8., true);
        ferry(
            &mut c,
            world,
            3,
            vec3(140., 22., 26.),
            4.8,
            5.6,
            vec3(0., 0., 20.),
            15.,
            0.,
        );
        deck(&mut c, world, 3, vec3(140., 23., 56.), 7., 8., false);
        ferry(
            &mut c,
            world,
            3,
            vec3(147., 23., 61.),
            5.,
            5.,
            vec3(10., 1., 8.),
            12.,
            0.,
        );
        deck(&mut c, world, 3, vec3(164., 24., 76.), 8., 8., true);
        deck(&mut c, world, 4, vec3(174., 24., 76.), 6., 6., false);
        ferry(
            &mut c,
            world,
            4,
            vec3(184., 24.8, 73.),
            3.8,
            3.8,
            vec3(0., 0., 6.),
            7.,
            0.,
        );
        deck(&mut c, world, 4, vec3(190., 25.4, 76.), 3.5, 3.5, false);
        ferry(
            &mut c,
            world,
            4,
            vec3(196., 26., 80.),
            3.8,
            3.8,
            vec3(0., 0., -8.),
            8.,
            2.,
        );
        deck(&mut c, world, 4, vec3(202., 26.6, 76.), 3.5, 3.5, false);
        ferry(
            &mut c,
            world,
            4,
            vec3(208., 27.2, 73.),
            3.8,
            3.8,
            vec3(0., 0., 6.),
            7.5,
            1.4,
        );
        ferry(
            &mut c,
            world,
            4,
            vec3(214., 27.8, 79.),
            3.8,
            3.8,
            vec3(0., 0., -6.),
            7.5,
            4.5,
        );
        deck(&mut c, world, 4, vec3(220., 28.4, 76.), 4., 4., false);
        deck(&mut c, world, 4, vec3(228., 29., 76.), 8., 8., true);
        deck(&mut c, world, 5, vec3(228., 29., 91.), 8., 12., false);
        deck(&mut c, world, 5, vec3(228., 29., 106.), 6., 6., false);
        deck(&mut c, world, 5, vec3(228., 29., 118.), 6., 16., false);
        c.nodes.last_mut().unwrap().offset = Vec3::Z * 3.;
        world
            .solids
            .push(Solid::new(vec3(228., 30.7, 118.), vec3(6.8, 0.9, 2.), NAVY));
        for x in [224.6, 231.4] {
            world
                .solids
                .push(Solid::new(vec3(x, 29.85, 118.), vec3(0.8, 1.7, 2.), NAVY));
        }
        deck(&mut c, world, 5, vec3(228., 29.8, 132.5), 5., 6., false);
        deck(&mut c, world, 5, vec3(228., 30., 143.), 8., 8., true);
        ferry(
            &mut c,
            world,
            6,
            vec3(217., 30., 143.),
            5.,
            5.,
            vec3(-12., 0., -8.),
            12.,
            0.,
        );
        deck(&mut c, world, 6, vec3(196., 30., 129.), 6., 6., false);
        ferry(
            &mut c,
            world,
            6,
            vec3(187., 30., 122.),
            5.,
            5.,
            vec3(-16., 2., -12.),
            15.5,
            0.,
        );
        deck(&mut c, world, 6, vec3(161., 33., 101.), 8., 8., true);
        ferry(
            &mut c,
            world,
            7,
            vec3(153., 33., 93.),
            4.5,
            4.5,
            vec3(0., 8., 0.),
            11.5,
            0.,
        );
        deck(&mut c, world, 7, vec3(146., 42., 93.), 6., 6., false);
        deck(&mut c, world, 7, vec3(137., 42., 93.), 3., 6., false);
        ferry(
            &mut c,
            world,
            7,
            vec3(128., 42., 90.),
            3.8,
            3.8,
            vec3(0., 0., 6.),
            6.5,
            0.,
        );
        deck(&mut c, world, 7, vec3(119., 43., 93.), 3.5, 6., false);
        ferry(
            &mut c,
            world,
            7,
            vec3(109., 43., 93.),
            5.,
            5.,
            vec3(0., 0., -12.),
            12.,
            0.,
        );
        deck(&mut c, world, 7, vec3(101., 44., 72.), 10., 12., true);
        world.course = c;
    }
}

#[derive(Clone, Debug)]
pub struct Run {
    pub next: usize,
    pub checkpoint: usize,
    pub practice: Option<usize>,
    pub finished: bool,
    pub failures: u32,
    pub elapsed: f32,
    pub started: bool,
}
impl Run {
    pub fn new(course: &Course, practice: Option<usize>) -> Self {
        let checkpoint = course.checkpoints[practice.unwrap_or(0)];
        Self {
            next: checkpoint + 1,
            checkpoint,
            practice,
            finished: false,
            failures: 0,
            elapsed: 0.,
            started: false,
        }
    }
    pub fn clock(&mut self, dt: f32, moving: bool) {
        self.started |= moving;
        if self.started && !self.finished {
            self.elapsed += dt;
        }
    }
    pub fn advance(&mut self, world: &World, player: &Player) -> bool {
        if self.finished || !world.course.nodes[self.next].reached(world, player) {
            return false;
        }
        let node = &world.course.nodes[self.next];
        if node.checkpoint {
            self.checkpoint = self.next;
        }
        self.next += 1;
        self.finished = self.next == world.course.nodes.len()
            || self
                .practice
                .is_some_and(|s| self.next > world.course.checkpoints[s + 1]);
        node.checkpoint
    }
    pub fn retry(&mut self) {
        self.next = self.checkpoint + 1;
        self.failures += 1;
        self.finished = false;
    }
    pub fn section(&self, course: &Course) -> usize {
        course.nodes[self.next.min(course.nodes.len() - 1)].section
    }
}
