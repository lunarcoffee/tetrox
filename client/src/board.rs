use crate::canvas::CanvasRenderer;
use crate::input::{Input, InputStates};
use gloo_timers::callback::{Interval, Timeout};
use tetrox::{
    field::{DefaultField, LineClear},
    tetromino::{SingleBag, SrsKickTable, SrsTetromino, Tetrio180KickTable},
};
use yew::{html, Component, Context, Html, KeyboardEvent, Properties};

pub enum BoardMessage {
    KeyPressed(KeyboardEvent),
    KeyReleased(KeyboardEvent),

    MoveLeft,
    MoveRight,
    MoveDown,

    MoveLeftAutoRepeat,
    MoveRightAutoRepeat,
    DasLeft,
    DasRight,
    ProjectDown,

    HardDrop,
    LockDelayDrop,

    FadeClearText,
}

#[derive(Clone, PartialEq, Properties)]
pub struct BoardProps {
    pub width: usize,
    pub height: usize,
    pub hidden: usize,
    pub queue_len: usize,
}

pub struct BoardTimers {
    gravity: Option<Interval>,
    lock_delay: Option<Timeout>,
    clear_type_animation: Option<Interval>,
}

const GRAVITY_DELAY: u32 = 1_000;
const LOCK_DELAY: u32 = 500;

impl BoardTimers {
    pub fn new() -> Self {
        BoardTimers {
            gravity: None,
            lock_delay: None,
            clear_type_animation: None,
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

    fn fade_clear_text(&mut self, ctx: &Context<Board>) {
        let link = ctx.link().clone();
        self.clear_type_animation = Some(Interval::new(20, move || {
            link.send_message(BoardMessage::FadeClearText);
        }));
    }

    fn cancel_clear_text(&mut self) { self.clear_type_animation.take().map(|timer| timer.cancel()); }
}

pub struct Board {
    bag: SingleBag<SrsTetromino>,
    field: DefaultField<SrsTetromino>,
    input_states: InputStates,
    canvas_renderer: CanvasRenderer,

    line_clear_type: Option<LineClear<SrsTetromino>>,
    line_clear_text: String,
    line_clear_text_opacity: f64,

    timers: BoardTimers,
    prev_lock_delay_actions: usize,
}

impl Board {
    fn reset(&mut self, ctx: &Context<Board>) {
        self.bag = SingleBag::new();

        let props = ctx.props();
        self.field = DefaultField::new(props.width, props.height, props.hidden, props.queue_len, &mut self.bag);

        self.input_states = InputStates::new();

        self.line_clear_type = None;
        self.line_clear_text = "".to_string();

        self.timers = BoardTimers::new();
        self.timers.reset_gravity(ctx);
    }

    fn update_clear_text(&mut self, ctx: &Context<Self>) {
        if let Some(clear_type) = self.line_clear_type.as_ref() {
            let n_lines = clear_type.n_lines();

            if n_lines > 0 || clear_type.spin().is_some() {
                let mini = clear_type.is_mini().then(|| "mini ").unwrap_or("");
                let spin = clear_type.spin().map(|_| "t-spin ").unwrap_or("");
                let n_text = ["", "single ", "double ", "triple ", "quad "][n_lines];
                let perfect_clear = self.field.is_empty().then(|| "\nperfect clear").unwrap_or("");

                self.line_clear_text = format!("{}{}{}{}", mini, spin, n_text, perfect_clear)
                    .trim()
                    .to_string();

                self.line_clear_text_opacity = 1.0;
                self.timers.fade_clear_text(ctx);
            }
        }
    }
}

impl Component for Board {
    type Message = BoardMessage;
    type Properties = BoardProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut bag = SingleBag::new();
        let props = ctx.props();
        let field = DefaultField::new(props.width, props.height, props.hidden, props.queue_len, &mut bag);

        Board {
            bag,
            field,
            input_states: InputStates::new(),
            canvas_renderer: CanvasRenderer::new(),

            line_clear_type: None,
            line_clear_text: "".to_string(),
            line_clear_text_opacity: 1.0,

            timers: BoardTimers::new(),
            prev_lock_delay_actions: 0,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        // handle input suppression first
        match msg {
            BoardMessage::MoveRight | BoardMessage::DasRight => {
                if self.input_states.is_pressed(Input::Left) {
                    self.input_states.set_suppressed(Input::Left);
                }
            }
            BoardMessage::MoveLeft | BoardMessage::DasLeft => {
                if self.input_states.is_pressed(Input::Right) {
                    self.input_states.set_suppressed(Input::Right);
                }
            }
            _ => {}
        }

        let to_true = |_| true;
        let to_false = |_| false;

        // primary input action
        let update = match msg {
            BoardMessage::KeyPressed(ref e) => match &e.key().to_lowercase()[..] {
                "arrowleft" => to_true(self.input_states.left_pressed(ctx)),
                "arrowright" => to_true(self.input_states.right_pressed(ctx)),
                "arrowdown" => to_true(self.input_states.soft_drop_pressed(ctx)),
                "arrowup" => to_true(
                    self.input_states
                        .set_pressed_with_action(Input::RotateCw, || self.field.try_rotate_cw(&SrsKickTable)),
                ),
                "s" => to_true(
                    self.input_states
                        .set_pressed_with_action(Input::RotateCcw, || self.field.try_rotate_ccw(&SrsKickTable)),
                ),
                "a" => to_true(
                    self.input_states
                        .set_pressed_with_action(Input::Rotate180, || self.field.try_rotate_180(&Tetrio180KickTable)),
                ),
                "d" => {
                    let result = self.field.swap_hold_piece(&mut self.bag);
                    if result {
                        self.timers.cancel_lock_delay();
                    }
                    result
                }
                " " => to_true(self.input_states.set_pressed_with_action(Input::HardDrop, || {
                    to_true(ctx.link().send_message(BoardMessage::HardDrop))
                })),
                "`" => to_true(self.reset(ctx)),
                _ => return false,
            },
            BoardMessage::KeyReleased(ref e) => {
                to_false(self.input_states.set_released(match &e.key().to_lowercase()[..] {
                    "arrowleft" => Input::Left,
                    "arrowright" => Input::Right,
                    "arrowdown" => Input::SoftDrop,
                    "arrowup" => Input::RotateCw,
                    "s" => Input::RotateCcw,
                    "a" => Input::Rotate180,
                    " " => Input::HardDrop,
                    _ => return false,
                }))
            }
            BoardMessage::MoveLeft => self.field.try_shift(0, -1),
            BoardMessage::MoveRight => self.field.try_shift(0, 1),
            BoardMessage::DasLeft => to_true(while self.field.try_shift(0, -1) {}),
            BoardMessage::DasRight => to_true(while self.field.try_shift(0, 1) {}),
            BoardMessage::MoveDown => self.field.try_shift(1, 0),
            BoardMessage::MoveLeftAutoRepeat => to_true(self.input_states.left_held(ctx)),
            BoardMessage::MoveRightAutoRepeat => to_true(self.input_states.right_held(ctx)),
            BoardMessage::ProjectDown => self.field.project_down(false),
            BoardMessage::HardDrop => {
                self.timers.reset_gravity(ctx);
                self.timers.cancel_lock_delay();
                self.prev_lock_delay_actions = 0;
                self.line_clear_type = Some(self.field.hard_drop(&mut self.bag));
                self.update_clear_text(ctx);
                true
            }
            // only lock if the piece is still touching the stack
            BoardMessage::LockDelayDrop => {
                if self.field.cur_piece_cannot_move_down() {
                    to_true(ctx.link().send_message(BoardMessage::HardDrop))
                } else {
                    false
                }
            }
            _ => true,
        };

        // messages for animations
        match msg {
            BoardMessage::FadeClearText => {
                if self.line_clear_text_opacity > 0.0 {
                    self.line_clear_text_opacity /= 1.0 / (self.line_clear_text_opacity - 1e-10);
                } else {
                    self.timers.cancel_clear_text();
                }
            }
            _ => {}
        }

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

        update
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {        
        html! {
            <div class="game">
                <div class="field-left-panel">
                    <div class="hold-piece">
                        { self.canvas_renderer.hold_piece_canvas() }
                    </div>
                    <div class="game-stats">
                        <p class="clear-type-text"
                           style={ format!("opacity: {};", self.line_clear_text_opacity) }>
                            { &self.line_clear_text }
                        </p>
                    </div>
                </div>
                <div class="field">
                    { self.canvas_renderer.field_canvas(&self.field, ctx) }
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
        }
        self.canvas_renderer.draw_hold_piece(&self.field);
        self.canvas_renderer.draw_next_queue(&self.field, &mut self.bag);
        self.canvas_renderer.draw_field(&self.field, first_render);
    }
}
