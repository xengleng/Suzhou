//! A tiny tweening helper.
//!
//! GPUI's `with_animation` is great for things that play once when they
//! appear. Panels, modals and switches also need to run *backwards* when they
//! close, from wherever they are, so those use a `Tween`: the view keeps one,
//! points it at a new target, and asks for another frame while it moves.

use std::time::{Duration, Instant};

use gpui::Window;

pub type Ease = fn(f32) -> f32;

pub fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

pub fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Overshoots a little and settles, like a soft spring.
pub fn ease_out_back(t: f32) -> f32 {
    let c1 = 1.4;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

/// Fast start, long gentle landing; the default for things sliding in.
pub fn ease_out_quint(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(5)
}

#[derive(Clone, Copy, Debug)]
pub struct Tween {
    from: f32,
    to: f32,
    start: Instant,
    duration: Duration,
    ease: Ease,
}

impl Tween {
    pub fn new(value: f32) -> Self {
        Tween {
            from: value,
            to: value,
            start: Instant::now(),
            duration: Duration::ZERO,
            ease: ease_out_cubic,
        }
    }

    pub fn value(&self) -> f32 {
        if self.duration.is_zero() {
            return self.to;
        }
        let t = (self.start.elapsed().as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0);
        self.from + (self.to - self.from) * (self.ease)(t)
    }

    pub fn target(&self) -> f32 {
        self.to
    }

    pub fn is_animating(&self) -> bool {
        !self.duration.is_zero() && self.start.elapsed() < self.duration
    }

    /// Moves toward `target` from wherever the tween is right now.
    pub fn animate_to(&mut self, target: f32, millis: u64, ease: Ease) {
        if (self.to - target).abs() < f32::EPSILON && !self.is_animating() {
            return;
        }
        self.from = self.value();
        self.to = target;
        self.start = Instant::now();
        self.duration = Duration::from_millis(millis);
        self.ease = ease;
    }

    pub fn set(&mut self, value: f32) {
        *self = Tween::new(value);
    }

    /// The value for this frame, asking for another frame while still moving.
    pub fn tick(&self, window: &mut Window) -> f32 {
        if self.is_animating() {
            window.request_animation_frame();
        }
        self.value()
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
