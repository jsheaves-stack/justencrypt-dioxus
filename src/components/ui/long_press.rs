use crate::util::sleep;
use dioxus::{core::Task, prelude::*};
use std::time::Duration;

const LONG_PRESS_DURATION: Duration = Duration::from_millis(500);

/// Press-and-hold gesture used by grid items (file/folder cards) to enter
/// multi-select without a right-click or modifier key. Wire `start`/`cancel`
/// to `onmousedown`/`onmouseup`/`onmouseleave`, and check `consume` at the
/// top of `onclick` to swallow the click that follows a completed long press.
#[derive(Clone, Copy)]
pub struct LongPress {
    is_pressed: Signal<bool>,
    was_triggered: Signal<bool>,
    task: Signal<Option<Task>>,
}

pub fn use_long_press() -> LongPress {
    LongPress {
        is_pressed: use_signal(|| false),
        was_triggered: use_signal(|| false),
        task: use_signal(|| None),
    }
}

impl LongPress {
    /// Begins the hold timer; calls `on_trigger` once if the press is still
    /// active after `LONG_PRESS_DURATION`.
    pub fn start(&mut self, mut on_trigger: impl FnMut() + 'static) {
        let mut is_pressed = self.is_pressed;
        let mut was_triggered = self.was_triggered;

        is_pressed.set(true);
        let handle = spawn(async move {
            sleep(LONG_PRESS_DURATION).await;
            if *is_pressed.read() {
                was_triggered.set(true);
                on_trigger();
            }
        });
        self.task.set(Some(handle));
    }

    /// Ends the gesture early (mouse released or left the element) before it fires.
    pub fn cancel(&mut self) {
        self.is_pressed.set(false);
        if let Some(handle) = self.task.take() {
            handle.cancel();
        }
    }

    /// Returns `true` (and resets) if the gesture already fired as a long
    /// press, so the caller's `onclick` can skip its normal single-click action.
    pub fn consume(&mut self) -> bool {
        if *self.was_triggered.read() {
            self.was_triggered.set(false);
            true
        } else {
            false
        }
    }
}
