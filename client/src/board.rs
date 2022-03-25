use std::collections::HashMap;

use crate::input::{Input, InputStates};
use strum::IntoEnumIterator;
use tetrox::{
    field::{DefaultField, Square},
    tetromino::{ExtendedSrsKickTable, SevenBag, SrsKickTable, Tetromino},
    Bag, Coords, PieceKind,
};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
use yew::{html, Component, Context, Html, KeyboardEvent, NodeRef};

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

pub struct Board {
    bag: SevenBag,
    field: DefaultField<Tetromino>,
    input_states: InputStates,

    field_canvas: NodeRef,
    hold_piece_canvas: NodeRef,
    next_queue_canvas: NodeRef,

    asset_cache: HashMap<Tetromino, HtmlImageElement>,
}

pub const LABEL_HEIGHT: usize = 30; // height of "hold" and "next" labels
pub const PIECE_HEIGHT: usize = 100; // height of hold/queue piece
pub const SIDE_BAR_WIDTH: usize = 170; // width of hold/queue panels
pub const SIDE_BAR_PADDING: usize = 6; // bottom padding of hold/queue panels
pub const SQUARE_MUL: usize = 32; // the size of each square on the field

impl Board {
    fn draw_hold_piece(&self) {
        let hp_h_px = (LABEL_HEIGHT + PIECE_HEIGHT + SIDE_BAR_PADDING) as f64;

        if let Some(canvas) = self.hold_piece_canvas.cast::<HtmlCanvasElement>() {
            let context = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();

            context.set_fill_style(&"black".into());
            context.clear_rect(0.0, 0.0, SIDE_BAR_WIDTH as f64, hp_h_px);

            // fill background
            context.set_stroke_style(&"black".into());
            context.set_global_alpha(0.6);
            context.fill_rect(0.0, 0.0, SIDE_BAR_WIDTH as f64, hp_h_px);

            // draw label
            context.set_fill_style(&"#bbb".into());
            context.set_global_alpha(1.0);
            context.set_font("18px 'IBM Plex Sans'");
            context.fill_text("hold", 8.0, 24.0).unwrap();

            if let Some(kind) = self.field.hold_piece() {
                self.draw_piece(kind, &context, SIDE_BAR_WIDTH / 2, LABEL_HEIGHT + PIECE_HEIGHT / 2)
            }
        }
    }

    fn draw_field(&self, first_render: bool) {
        // field width and height in squares
        let fw = self.field.width() as f64;
        let fh = self.field.height() as f64;

        // units in pixels
        let fw_px = SQUARE_MUL as f64 * fw;
        let fh_px = SQUARE_MUL as f64 * fh;
        let fhidden_end_px = (self.field.hidden() * SQUARE_MUL) as f64; // end of board hidden area

        if let Some(canvas) = self.field_canvas.cast::<HtmlCanvasElement>() {
            if first_render {
                canvas.focus().unwrap();
            }

            let context = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();

            context.set_fill_style(&"black".into());
            context.clear_rect(0.0, 0.0, fw_px, fh_px);

            // fill background
            context.set_global_alpha(0.6);
            context.fill_rect(0.0, fhidden_end_px, fw_px, fh_px);

            context.set_stroke_style(&"#555".into());
            context.set_global_alpha(0.3);

            // vertical grid lines
            for col in 1..self.field.width() {
                context.begin_path();
                context.move_to((col * SQUARE_MUL) as f64, fhidden_end_px);
                context.line_to((col * SQUARE_MUL) as f64, fh_px);
                context.stroke();
            }

            // horizontal grid lines (only for non-hidden board area)
            for row in self.field.hidden() + 1..self.field.height() {
                context.begin_path();
                context.move_to(0.0, (row * SQUARE_MUL) as f64);
                context.line_to(fh_px, (row * SQUARE_MUL) as f64);
                context.stroke();
            }

            let shadow_piece = self.field.shadow_piece();
            for Coords(row, col) in shadow_piece.coords() {
                self.draw_square(
                    &shadow_piece.kind(),
                    &context,
                    *row as usize * SQUARE_MUL,
                    *col as usize * SQUARE_MUL,
                );
            }

            context.set_global_alpha(1.0);
            for (row, line) in self.field.lines().iter().enumerate() {
                for (col, square) in line.squares().iter().enumerate() {
                    if let Square::Filled(kind) = square {
                        self.draw_square(kind, &context, row * SQUARE_MUL, col * SQUARE_MUL);
                    }
                }
            }
        }
    }

    fn draw_next_queue(&mut self) {
        // total height of queue in pixels
        let nq_h_px = (LABEL_HEIGHT + PIECE_HEIGHT * self.field.queue_len() + SIDE_BAR_PADDING) as f64;

        if let Some(canvas) = self.next_queue_canvas.cast::<HtmlCanvasElement>() {
            let context = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();

            context.set_fill_style(&"black".into());
            context.clear_rect(0.0, 0.0, SIDE_BAR_WIDTH as f64, nq_h_px);

            // fill background
            context.set_stroke_style(&"black".into());
            context.set_global_alpha(0.6);
            context.fill_rect(0.0, 0.0, SIDE_BAR_WIDTH as f64, nq_h_px);

            // draw label
            context.set_fill_style(&"#bbb".into());
            context.set_global_alpha(1.0);
            context.set_font("18px 'IBM Plex Sans'");
            context.fill_text("next", 8.0, 24.0).unwrap();

            let queue = self
                .bag
                .peek()
                .take(self.field.queue_len())
                .cloned()
                .collect::<Vec<_>>();

            for (nth, kind) in queue.iter().enumerate() {
                self.draw_piece(
                    *kind,
                    &context,
                    SIDE_BAR_WIDTH / 2,
                    LABEL_HEIGHT + PIECE_HEIGHT * (nth + 1) - PIECE_HEIGHT / 2,
                )
            }
        }
    }

    fn reset(&mut self) -> bool {
        self.bag = SevenBag::new();
        self.field = DefaultField::new(10, 40, 20, 5, &mut self.bag);
        self.input_states = InputStates::new();
        true
    }

    fn draw_piece(&self, kind: Tetromino, context: &CanvasRenderingContext2d, x_offset: usize, y_offset: usize) {
        let base_coords = kind
            .spawn_offsets()
            .into_iter()
            .map(|Coords(row, col)| Coords(row * SQUARE_MUL as i32, col * SQUARE_MUL as i32))
            .collect();

        let offset = Coords(y_offset as i32, x_offset as i32);
        let final_coords = Self::center_coords_around_origin(base_coords)
            .into_iter()
            .map(|c| c + offset);

        for Coords(row, col) in final_coords {
            self.draw_square(&kind, context, row as usize, col as usize);
        }
    }

    // draw a square at the given coords on a canvas
    fn draw_square(&self, kind: &Tetromino, context: &CanvasRenderingContext2d, row: usize, col: usize) {
        context
            .draw_image_with_html_image_element_and_dw_and_dh(
                &self.asset_cache.get(kind).unwrap(),
                col as f64,
                row as f64,
                SQUARE_MUL as f64,
                SQUARE_MUL as f64,
            )
            .unwrap();
    }

    // transforms `coords` so that if a square is drawn from each set of coords, the entire image will be centered
    // around the origin
    fn center_coords_around_origin(coords: Vec<Coords>) -> Vec<Coords> {
        let min_col = coords.iter().min_by_key(|Coords(_, col)| col).unwrap().1;
        let max_col = coords.iter().max_by_key(|Coords(_, col)| col).unwrap().1;
        let min_row = coords.iter().min_by_key(|Coords(row, _)| row).unwrap().0;
        let max_row = coords.iter().max_by_key(|Coords(row, _)| row).unwrap().0;

        let offset = Coords((max_row + min_row) / 2, (max_col + min_col) / 2);
        coords
            .into_iter()
            // (0, 0) is not the center since images are drawn from the top-left corner
            // the actual center is half a `SQUARE_MUL` away in both directions
            .map(|c| c - offset - Coords(SQUARE_MUL as i32 / 2, SQUARE_MUL as i32 / 2))
            .collect()
    }

    fn populate_asset_cache() -> HashMap<Tetromino, HtmlImageElement> {
        Tetromino::iter()
            .map(|kind| {
                let field_square_mul = SQUARE_MUL as u32;
                let image = HtmlImageElement::new_with_width_and_height(field_square_mul, field_square_mul).unwrap();
                let asset_src = format!("assets/skins/{}/{}.png", crate::SKIN_NAME, kind.asset_name());
                image.set_src(&asset_src);
                (kind, image)
            })
            .collect()
    }
}

impl Component for Board {
    type Message = BoardMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let mut bag = SevenBag::new();
        let field = DefaultField::new(10, 40, 20, 5, &mut bag);

        Board {
            bag,
            field,
            input_states: InputStates::new(),
            field_canvas: NodeRef::default(),
            hold_piece_canvas: NodeRef::default(),
            next_queue_canvas: NodeRef::default(),
            asset_cache: Self::populate_asset_cache(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            BoardMessage::KeyPressed(e) => match &e.key().to_lowercase()[..] {
                "arrowleft" => self.input_states.left_pressed(ctx),
                "arrowright" => self.input_states.right_pressed(ctx),
                "arrowdown" => self.input_states.soft_drop_pressed(ctx),
                "arrowup" => self
                    .input_states
                    .set_pressed_with_action(Input::RotateCw, || self.field.try_rotate_cw(&SrsKickTable)),
                "s" => self
                    .input_states
                    .set_pressed_with_action(Input::RotateCcw, || self.field.try_rotate_ccw(&SrsKickTable)),
                "a" => self
                    .input_states
                    .set_pressed_with_action(Input::Rotate180, || self.field.try_rotate_180(&ExtendedSrsKickTable)),
                "d" => self.field.swap_hold_piece(&mut self.bag),
                " " => self
                    .input_states
                    .set_pressed_with_action(Input::HardDrop, || self.field.hard_drop(&mut self.bag)),
                "`" => self.reset(),
                _ => return false,
            },
            BoardMessage::KeyReleased(e) => self.input_states.set_released(match &e.key().to_lowercase()[..] {
                "arrowleft" => Input::Left,
                "arrowright" => Input::Right,
                "arrowdown" => Input::SoftDrop,
                "arrowup" => Input::RotateCw,
                "s" => Input::RotateCcw,
                "a" => Input::Rotate180,
                " " => Input::HardDrop,
                _ => return false,
            }),
            BoardMessage::MoveLeft => {
                if self.input_states.is_pressed(Input::Right) {
                    self.input_states.set_suppressed(Input::Right);
                }
                self.field.try_shift(0, -1)
            }
            BoardMessage::MoveRight => {
                if self.input_states.is_pressed(Input::Left) {
                    self.input_states.set_suppressed(Input::Left);
                }
                self.field.try_shift(0, 1)
            }
            BoardMessage::DasLeft => {
                if self.input_states.is_pressed(Input::Right) {
                    self.input_states.set_suppressed(Input::Right);
                }
                while self.field.try_shift(0, -1) {}
                true
            }
            BoardMessage::DasRight => {
                if self.input_states.is_pressed(Input::Left) {
                    self.input_states.set_suppressed(Input::Left);
                }
                while self.field.try_shift(0, 1) {}
                true
            }
            BoardMessage::MoveDown => self.field.try_shift(1, 0),
            BoardMessage::MoveLeftAutoRepeat => self.input_states.left_held(ctx),
            BoardMessage::MoveRightAutoRepeat => self.input_states.right_held(ctx),
            BoardMessage::ProjectDown => self.field.project_down(),
        };
        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let link = ctx.link();
        let key_pressed_callback = link.callback(|e| BoardMessage::KeyPressed(e));
        let key_released_callback = link.callback(|e| BoardMessage::KeyReleased(e));

        html! {
            <div class="game">
                <div class="hold-piece">
                    <canvas ref={ self.hold_piece_canvas.clone() }
                            class="hold-piece-canvas"
                            width={ SIDE_BAR_WIDTH.to_string() }
                            height={ (LABEL_HEIGHT + PIECE_HEIGHT + SIDE_BAR_PADDING).to_string() }>
                    </canvas>
                </div>
                <div class="field">
                    <canvas ref={ self.field_canvas.clone() }
                            class="field-canvas"
                            // to hide the hidden area of the board
                            style={ format!("margin-top: -{}px;", SQUARE_MUL * self.field.hidden()) }
                            tabindex="0"
                            onkeydown={ key_pressed_callback }
                            onkeyup={ key_released_callback }
                            width={ (SQUARE_MUL * self.field.width()).to_string() }
                            height={ (SQUARE_MUL * self.field.height()).to_string() }>
                    </canvas>
                </div>
                <div class="next-queue">
                    <canvas ref={ self.next_queue_canvas.clone() }
                            class="next-queue-canvas"
                            width={ SIDE_BAR_WIDTH.to_string() }
                            height={
                                (LABEL_HEIGHT + PIECE_HEIGHT * self.field.queue_len() + SIDE_BAR_PADDING).to_string()
                            }>
                    </canvas>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        self.draw_hold_piece();
        self.draw_field(first_render);
        self.draw_next_queue();
    }
}