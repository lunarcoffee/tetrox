use std::{cell::RefCell, mem};

use gloo_timers::callback::Timeout;
use sycamore::prelude::{create_effect, create_signal, use_scope_status, ReadSignal, Scope, Signal};

// effect executed when the given `timer` finishes
// if `op` returns true, the timer will start again (making a loop)
pub fn create_timer_finish_effect<'a>(cx: Scope<'a>, timer: &'a ReadSignal<Timer>, mut op: impl FnMut() -> bool + 'a) {
    create_effect(cx, move || {
        if timer.get().is_finished() && op() {
            timer.get().start();
        }
    });
}

// a resettable timer that waits for a timeout and sets a flag upon completion
pub struct Timer<'a>(RefCell<TimerInner<'a>>);

struct TimerInner<'a> {
    cx: Scope<'a>,

    duration: u32,
    timeout: Option<Timeout>,
    is_finished: &'a Signal<bool>,
}

impl<'a> Timer<'a> {
    pub fn new(cx: Scope<'a>, duration: u32) -> Self {
        Timer(RefCell::new(TimerInner {
            cx,

            duration,
            timeout: None,
            is_finished: create_signal(cx, false),
        }))
    }

    // this value is reactive and should be used to perform an action on completion of the timeout
    pub fn is_finished(&self) -> bool { *self.0.borrow().is_finished.get() }

    // run the timer, setting the `is_finished` signal to true when the `duration` has elapsed
    pub fn start(&self) {
        self.stop();

        let scope_alive = use_scope_status(self.0.borrow().cx);
        let is_finished = self.0.borrow().is_finished.clone();

        // SAFETY: transmuting from 'a to 'static lets this be used in the timeout
        // this is safe as we check if the scope is alive before calling the closure
        let is_finished = unsafe { mem::transmute::<_, &'static Signal<bool>>(is_finished) };

        let timeout = Timeout::new(self.0.borrow().duration, move || {
            if *scope_alive.get() {
                is_finished.set(true);
            }
        });
        self.0.borrow_mut().timeout = Some(timeout);
    }

    // stop any currently running timer and mark it as unfinished, effectively resetting it
    pub fn stop(&self) {
        self.0.borrow_mut().timeout.take().map(|t| t.cancel());
        self.0.borrow().is_finished.set(false);
    }
}
