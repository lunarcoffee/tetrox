use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use yew::Context;

use crate::board::{Board, BoardMessage};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Animation {
    LineClearText,
    PerfectClearText,
    BackToBackText,
    ComboText,
}

pub enum AnimationData {
    Float2(f64, f64),
}

impl AnimationData {
    pub fn extract_float2(&self) -> (f64, f64) {
        match self {
            AnimationData::Float2(a, b) => (*a, *b),
        }
    }
}

// if `None` is returned, the animation will be stopped
// otherwise, the state will be assigned the updated value
pub trait AnimationUpdater = Fn(&AnimationData) -> Option<AnimationData>;

pub struct AnimationState {
    active_animations: Rc<RefCell<HashSet<Animation>>>,
    animation_state: HashMap<Animation, AnimationData>,
    animation_updaters: HashMap<Animation, Box<dyn AnimationUpdater>>,
}

impl AnimationState {
    pub fn new() -> Self {
        AnimationState {
            active_animations: Rc::new(RefCell::new(HashSet::new())),
            animation_state: HashMap::new(),
            animation_updaters: HashMap::new(),
        }
    }

    // get all animations that are to be updated every tick
    pub fn get_active(&self) -> Rc<RefCell<HashSet<Animation>>> { self.active_animations.clone() }

    // gets data associated with an animation (e.g element opacity)
    pub fn get_state(&self, animation: Animation) -> Option<&AnimationData> { self.animation_state.get(&animation) }

    pub fn extract_state<T>(
        &self,
        animation: Animation,
        extract_func: impl FnOnce(&AnimationData) -> T,
        default: T,
    ) -> T {
        self.get_state(animation).map(|a| extract_func(a)).unwrap_or(default)
    }

    pub fn set_state(&mut self, animation: Animation, data: AnimationData) {
        self.animation_state.insert(animation, data);
    }

    // set an updater to be called every frame for an animation
    pub fn set_updater(&mut self, animation: Animation, updater: impl AnimationUpdater + 'static) {
        self.animation_updaters.insert(animation, Box::new(updater));
    }

    // set the updater for the given animation to be called every tick
    pub fn start_animation(&mut self, animation: Animation) { self.active_animations.borrow_mut().insert(animation); }

    pub fn stop_animation(&mut self, animation: Animation) { self.active_animations.borrow_mut().remove(&animation); }

    // update the animation's state with the assigned updater
    pub fn tick(&mut self, ctx: &Context<Board>, animation: Animation) {
        let updater = self.animation_updaters.get(&animation).unwrap();
        match updater(self.get_state(animation).unwrap()) {
            Some(updated) => self.set_state(animation, updated),
            _ => ctx.link().send_message(BoardMessage::StopAnimation(animation)),
        }
    }
}
