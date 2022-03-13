#![feature(stmt_expr_attributes)]

use input::{Input, InputStates};
use tetrox::{
    field::{DefaultField, Square},
    tetromino::{ExtendedSrsKickTable, SevenBag, SrsKickTable, Tetromino},
    Coords, PieceKind,
};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
use yew::{html, Component, Context, Html, KeyboardEvent, NodeRef};

mod input;

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
}

pub struct BoardModel {
    bag: SevenBag,
    field: DefaultField<Tetromino>,
    canvas: NodeRef,
    input_states: InputStates,
}

impl BoardModel {
    // draw a square at the given coords on the canvas
    fn draw_square(&self, kind: &Tetromino, context: &CanvasRenderingContext2d, row: usize, col: usize) {
        let image_elem = HtmlImageElement::new_with_width_and_height(32, 32).unwrap();
        let asset_src = format!("assets/{}.png", kind.asset_name());
        image_elem.set_src(&asset_src);

        context
            .draw_image_with_html_image_element_and_dw_and_dh(
                &image_elem,
                32.0 * col as f64,
                32.0 * row as f64,
                32.0,
                32.0,
            )
            .unwrap();
    }

    fn reset(&mut self) -> bool {
        self.bag = SevenBag::new();
        self.field = DefaultField::new(10, 40, 20, &mut self.bag);
        self.input_states = InputStates::new();
        true
    }
}

impl Component for BoardModel {
    type Message = BoardMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let mut bag = SevenBag::new();
        let field = DefaultField::new(10, 40, 20, &mut bag);

        BoardModel {
            bag,
            field,
            canvas: NodeRef::default(),
            input_states: InputStates::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            BoardMessage::KeyPressed(e) => match &e.key()[..] {
                "ArrowLeft" => self.input_states.left_pressed(ctx),
                "ArrowRight" => self.input_states.right_pressed(ctx),
                "ArrowDown" => self.input_states.soft_drop_pressed(ctx),
                "ArrowUp" => self
                    .input_states
                    .set_pressed_with_action(Input::RotateCw, || self.field.try_rotate_cw(&SrsKickTable)),
                "s" => self
                    .input_states
                    .set_pressed_with_action(Input::RotateCcw, || self.field.try_rotate_ccw(&SrsKickTable)),
                "a" => self
                    .input_states
                    .set_pressed_with_action(Input::Rotate180, || self.field.try_rotate_180(&ExtendedSrsKickTable)),
                " " => self
                    .input_states
                    .set_pressed_with_action(Input::HardDrop, || self.field.hard_drop(&mut self.bag)),
                "`" => self.reset(),
                _ => return false,
            },
            BoardMessage::KeyReleased(e) => match &e.key()[..] {
                "ArrowLeft" => self.input_states.left_released(),
                "ArrowRight" => self.input_states.right_released(),
                "ArrowDown" => self.input_states.soft_drop_released(),
                "ArrowUp" => self.input_states.set_released(Input::RotateCw),
                "s" => self.input_states.set_released(Input::RotateCcw),
                "a" => self.input_states.set_released(Input::Rotate180),
                " " => self.input_states.set_released(Input::HardDrop),
                _ => return false,
            },
            BoardMessage::MoveLeft => {
                if !self.input_states.is_pressed(Input::Right) {
                    self.field.try_shift(0, -1)
                } else {
                    false
                }
            }
            BoardMessage::MoveRight => {
                if !self.input_states.is_pressed(Input::Left) {
                    self.field.try_shift(0, 1)
                } else {
                    false
                }
            }
            BoardMessage::MoveDown => self.field.try_shift(1, 0),
            BoardMessage::MoveLeftAutoRepeat => self.input_states.left_held(ctx),
            BoardMessage::MoveRightAutoRepeat => self.input_states.right_held(ctx),
            BoardMessage::DasLeft => {
                while self.field.try_shift(0, -1) {}
                true
            }
            BoardMessage::DasRight => {
                while self.field.try_shift(0, 1) {}
                true
            }
            BoardMessage::ProjectDown => self.field.project_down(),
        };
        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let link = ctx.link();
        let key_pressed_callback = link.callback(|e| BoardMessage::KeyPressed(e));
        let key_released_callback = link.callback(|e| BoardMessage::KeyReleased(e));

        html! {
            <div class="field">
                <canvas ref={ self.canvas.clone() }
                        class="field-canvas"
                        tabindex="0"
                        onkeydown={ key_pressed_callback }
                        onkeyup={ key_released_callback }
                        width={ "320" }
                        height={ "1280" }>
                </canvas>
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if let Some(canvas) = self.canvas.cast::<HtmlCanvasElement>() {
            if first_render {
                canvas.focus().unwrap();
            }

            let context = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();

            context.clear_rect(0.0, 0.0, 32.0 * 10.0, 32.0 * 40.0);

            context.set_stroke_style(&"black".into());
            context.set_global_alpha(0.6);
            context.fill_rect(0.0, 32.0 * 20.0, 32.0 * 10.0, 32.0 * 40.0);

            context.set_stroke_style(&"#555".into());
            
            // draw grid crosshair marks
            for col_n in 1..=9 {
                for row_n in 21..=39 {
                    let row = col_n as f64 * 32.0;
                    let col = row_n as f64 * 32.0;

                    context.begin_path();
                    context.move_to(row - 6.0, col);
                    context.line_to(row + 6.0, col);
                    context.move_to(row, col - 6.0);
                    context.line_to(row, col + 6.0);
                    context.stroke();
                }
            }

            // draw grid lines
            context.set_global_alpha(0.3);
            for col in 1..=9 {
                context.begin_path();
                context.move_to(col as f64 * 32.0, 32.0 * 20.0);
                context.line_to(col as f64 * 32.0, 32.0 * 40.0);
                context.stroke();
            }
            for row in 21..=39 {
                context.begin_path();
                context.move_to(0.0, row as f64 * 32.0);
                context.line_to(32.0 * 40.0, row as f64 * 32.0);
                context.stroke();
            }

            let shadow_piece = self.field.shadow_piece();
            for Coords(row, col) in shadow_piece.coords() {
                self.draw_square(&shadow_piece.kind(), &context, *row as usize, *col as usize);
            }

            context.set_global_alpha(1.0);
            for (row, line) in self.field.lines().iter().enumerate() {
                for (col, square) in line.squares().iter().enumerate() {
                    match square {
                        Square::Filled(kind) => self.draw_square(kind, &context, row, col),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn main() { yew::start_app::<BoardModel>(); }
