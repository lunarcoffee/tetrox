use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use crate::animation::{Animation, AnimationState};
use crate::canvas::CanvasRenderer;
use crate::config::ReadOnlyConfig;
use crate::game_stats::GameStatsDrawer;
use crate::input::{Input, InputStates};
use gloo_timers::callback::{Interval, Timeout};
use tetrox::{
    field::DefaultField,
    tetromino::{SingleBag, SrsKickTable, SrsTetromino, Tetrio180KickTable},
};
use web_sys::HtmlElement;
use yew::{html, Component, Context, Html, KeyboardEvent, NodeRef, Properties};

#[derive(PartialEq, Properties)]
pub struct BoardProps {
    pub config: ReadOnlyConfig,
}

pub enum BoardMessage {
    KeyPressed(KeyboardEvent),
    KeyReleased(KeyboardEvent),

    MoveLeft,
    MoveRight,
    MoveDown,

    // repeating movement messages
    MoveLeftAutoRepeat,
    MoveRightAutoRepeat,
    DasLeft,
    DasRight,
    ProjectDown,

    RotateCw,
    RotateCcw,
    Rotate180,

    SwapHoldPiece,
    HardDrop,
    LockDelayDrop,

    TickAnimation(Animation),
    StopAnimation(Animation),

    Reset,
}

pub struct BoardTimers {
    gravity: Option<Interval>,
    lock_delay: Option<Timeout>,
    animation_loop: Option<Interval>,
}

const GRAVITY_DELAY: u32 = 1_000;
const LOCK_DELAY: u32 = 500;

impl BoardTimers {
    fn new(ctx: &Context<Board>, animations: Rc<RefCell<HashSet<Animation>>>) -> Self {
        // tick each animation once every frame at about 60 fps
        let link = ctx.link().clone();
        let animation_loop = Some(Interval::new(17, move || {
            // clone animations to avoid double borrow on animation stop
            let active_animations = animations.borrow().iter().cloned().collect::<Vec<_>>();
            for animation in active_animations {
                link.send_message(BoardMessage::TickAnimation(animation));
            }
        }));

        BoardTimers {
            gravity: None,
            lock_delay: None,
            animation_loop,
        }
    }

    fn reset_gravity(&mut self, ctx: &Context<Board>) {
        let link = ctx.link().clone();
        self.gravity = Some(Interval::new(GRAVITY_DELAY, move || {
            link.send_message(BoardMessage::MoveDown);
        }));
    }

    fn reset_lock_delay(&mut self, ctx: &Context<Board>) {
        let link = ctx.link().clone();
        self.lock_delay = Some(Timeout::new(LOCK_DELAY, move || {
            link.send_message(BoardMessage::LockDelayDrop);
        }));
    }

    fn cancel_lock_delay(&mut self) { self.lock_delay.take().map(|timer| timer.cancel()); }
}

impl Drop for BoardTimers {
    fn drop(&mut self) { self.animation_loop.take().unwrap().cancel(); }
}

pub struct Board {
    bag: SingleBag<SrsTetromino>,
    field: DefaultField<SrsTetromino>,
    input_states: InputStates,

    canvas_renderer: CanvasRenderer,
    game_stats: GameStatsDrawer,

    timers: BoardTimers,
    prev_lock_delay_actions: usize,
    animation_state: AnimationState,

    game_div: NodeRef, // used to set focus after rendering
}

impl Board {
    fn process_key_press(&mut self, ctx: &Context<Self>, e: &KeyboardEvent) -> bool {
        let inputs = &mut self.input_states;
        match &e.key().to_lowercase()[..] {
            "arrowleft" => inputs.left_pressed(ctx),
            "arrowright" => inputs.right_pressed(ctx),
            "arrowdown" => inputs.soft_drop_pressed(ctx),
            "arrowup" => inputs.set_pressed_msg(Input::RotateCw, ctx, BoardMessage::RotateCw),
            "s" => inputs.set_pressed_msg(Input::RotateCcw, ctx, BoardMessage::RotateCcw),
            "a" => inputs.set_pressed_msg(Input::Rotate180, ctx, BoardMessage::Rotate180),
            "d" => ctx.link().send_message(BoardMessage::SwapHoldPiece),
            " " => inputs.set_pressed_msg(Input::HardDrop, ctx, BoardMessage::HardDrop),
            "`" => ctx.link().send_message(BoardMessage::Reset),
            _ => return false,
        }
        true
    }

    fn process_key_release(&mut self, e: &KeyboardEvent) -> bool {
        self.input_states.set_released(match &e.key().to_lowercase()[..] {
            "arrowleft" => Input::Left,
            "arrowright" => Input::Right,
            "arrowdown" => Input::SoftDrop,
            "arrowup" => Input::RotateCw,
            "s" => Input::RotateCcw,
            "a" => Input::Rotate180,
            " " => Input::HardDrop,
            _ => return false,
        });
        false
    }

    fn process_lock_delay(&mut self, ctx: &Context<Self>, msg: BoardMessage) {
        // activate lock delay after the piece touches the stack while falling
        match msg {
            BoardMessage::MoveLeft | BoardMessage::MoveRight | BoardMessage::MoveDown | BoardMessage::ProjectDown => {
                if self.field.cur_piece_cannot_move_down() {
                    // only reset the lock delay the first time the piece touches the stack
                    if self.field.actions_since_lock_delay().is_none() {
                        self.timers.reset_lock_delay(ctx);
                    }
                    self.field.activate_lock_delay();
                }
            }
            _ => {}
        }

        if let Some(n_actions_now) = self.field.actions_since_lock_delay() {
            // reset the lock delay if a lock delay resetting action occurred (e.g. successful movement)
            if n_actions_now > self.prev_lock_delay_actions {
                self.timers.reset_lock_delay(ctx);
                self.prev_lock_delay_actions = n_actions_now;

                // cap how many such actions can occur
                if n_actions_now == 30 {
                    ctx.link().send_message(BoardMessage::HardDrop);
                }
            }
        }
    }

    fn reset(&mut self, ctx: &Context<Board>) {
        self.bag = SingleBag::new();

        let config = &ctx.props().config;
        self.field = DefaultField::new(
            config.field_width,
            config.field_height,
            config.field_hidden,
            config.queue_len,
            &mut self.bag,
        );
        self.input_states = InputStates::new(config.clone());

        self.game_stats = GameStatsDrawer::new();

        self.animation_state = AnimationState::new();
        self.timers = BoardTimers::new(ctx, self.animation_state.get_active());
        self.timers.reset_gravity(ctx);
    }
}

impl Component for Board {
    type Message = BoardMessage;
    type Properties = BoardProps;

    fn create(ctx: &Context<Self>) -> Self {
        let config = &ctx.props().config;

        let mut bag = SingleBag::new();
        let field = DefaultField::new(
            config.field_width,
            config.field_height,
            config.field_hidden,
            config.queue_len,
            &mut bag,
        );
        let animation_state = AnimationState::new();

        Board {
            bag,
            field,
            input_states: InputStates::new(config.clone()),

            canvas_renderer: CanvasRenderer::new(config.clone()),
            game_stats: GameStatsDrawer::new(),

            timers: BoardTimers::new(ctx, animation_state.get_active()),
            animation_state,
            prev_lock_delay_actions: 0,

            game_div: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        // handle input suppression first
        match msg {
            BoardMessage::MoveRight | BoardMessage::DasRight if self.input_states.is_pressed(Input::Left) => {
                self.input_states.set_suppressed(Input::Left)
            }
            BoardMessage::MoveLeft | BoardMessage::DasLeft if self.input_states.is_pressed(Input::Right) => {
                self.input_states.set_suppressed(Input::Right);
            }
            _ => {}
        }

        match msg {
            BoardMessage::KeyPressed(ref e) => return self.process_key_press(ctx, e),
            BoardMessage::KeyReleased(ref e) => return self.process_key_release(e),
            _ => {}
        };

        // avoids clutter from making every match arm a block with `true` or `false` at the end
        let to_true = |_| true;
        let to_false = |_| false;

        // action messages
        let update = match msg {
            BoardMessage::MoveLeft => self.field.try_shift(0, -1),
            BoardMessage::MoveRight => self.field.try_shift(0, 1),
            BoardMessage::MoveDown => self.field.try_shift(1, 0),

            BoardMessage::MoveLeftAutoRepeat => to_true(self.input_states.left_held(ctx)),
            BoardMessage::MoveRightAutoRepeat => to_true(self.input_states.right_held(ctx)),
            BoardMessage::DasLeft => to_true(while self.field.try_shift(0, -1) {}),
            BoardMessage::DasRight => to_true(while self.field.try_shift(0, 1) {}),
            BoardMessage::ProjectDown => self.field.project_down(),

            BoardMessage::RotateCw => self.field.try_rotate_cw(&SrsKickTable),
            BoardMessage::RotateCcw => self.field.try_rotate_ccw(&SrsKickTable),
            BoardMessage::Rotate180 => self.field.try_rotate_180(&Tetrio180KickTable),

            BoardMessage::SwapHoldPiece => {
                let result = self.field.swap_hold_piece(&mut self.bag);
                if result {
                    self.timers.cancel_lock_delay();
                }
                result
            }
            BoardMessage::HardDrop => {
                self.timers.reset_gravity(ctx);
                self.timers.cancel_lock_delay();
                self.prev_lock_delay_actions = 0;

                let clear_type = self.field.hard_drop(&mut self.bag);
                self.game_stats.set_clear_type(&mut self.animation_state, clear_type);

                true
            }
            BoardMessage::LockDelayDrop => {
                // only lock if the piece is still touching the stack
                let touching_stack = self.field.cur_piece_cannot_move_down();
                if touching_stack {
                    ctx.link().send_message(BoardMessage::HardDrop);
                }
                touching_stack
            }

            BoardMessage::TickAnimation(animation) => to_true(self.animation_state.tick(ctx, animation)),
            BoardMessage::StopAnimation(animation) => to_false(self.animation_state.stop_animation(animation)),

            BoardMessage::Reset => to_true(self.reset(ctx)),
            _ => false,
        };

        // process lock delay after movement actions (which may affect it)
        self.process_lock_delay(ctx, msg);

        update
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let config = &ctx.props().config;

        let field_changed = config.field_width != self.field.width()
            || config.field_height != self.field.height()
            || config.field_hidden != self.field.hidden()
            || config.queue_len != self.field.queue_len();

        self.input_states.update_config(config.clone());
        self.canvas_renderer.update_config(config.clone());

        if field_changed {
            self.reset(ctx);
        }
        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let link = ctx.link();
        let key_pressed_callback = link.callback(|e| BoardMessage::KeyPressed(e));
        let key_released_callback = link.callback(|e| BoardMessage::KeyReleased(e));

        html! {
            <div ref={ self.game_div.clone() }
                 class="game"
                 tabindex="0"
                 onkeydown={ key_pressed_callback }
                 onkeyup={ key_released_callback }>
                <div class="field-left-panel">
                    <div class="hold-piece">
                        { self.canvas_renderer.hold_piece_canvas() }
                    </div>
                    { self.game_stats.get_html(&self.animation_state) }
                </div>
                <div class="field">
                    { self.canvas_renderer.field_canvas(&self.field) }
                </div>
                <div class="next-queue">
                    { self.canvas_renderer.next_queue_canvas(&self.field) }
                </div>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.timers.reset_gravity(ctx);
            self.game_div.cast::<HtmlElement>().unwrap().focus().unwrap();
        }
        self.canvas_renderer.draw_hold_piece(&self.field);
        self.canvas_renderer.draw_next_queue(&self.field, &mut self.bag);
        self.canvas_renderer.draw_field(&self.field, first_render);
    }
}
