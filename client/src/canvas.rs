use std::collections::HashMap;

use tetrox::{
    field::{DefaultField, Square},
    tetromino::{SingleBag, SrsTetromino},
    Bag, Coords, PieceKind,
};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
use yew::{html, Html, NodeRef};

use crate::config::ReadOnlyConfig;

pub struct CanvasRenderer {
    hold_piece_canvas: NodeRef,
    field_canvas: NodeRef,
    next_queue_canvas: NodeRef,

    asset_cache: HashMap<SrsTetromino, HtmlImageElement>, // cache image assets for performance

    config: ReadOnlyConfig,
}

pub const SQUARE_MUL: usize = 36; // the size of each square on the field

pub const LABEL_HEIGHT: usize = 30; // height of "hold" and "next" labels
pub const PIECE_HEIGHT: usize = SQUARE_MUL * 3; // height of hold/queue piece

pub const SIDE_BAR_WIDTH: usize = SQUARE_MUL * 5; // width of hold/queue panels
pub const SIDE_BAR_PADDING: usize = SQUARE_MUL / 6; // bottom padding of hold/queue panels

impl CanvasRenderer {
    pub fn new(config: ReadOnlyConfig) -> Self {
        let mut renderer = CanvasRenderer {
            hold_piece_canvas: NodeRef::default(),
            field_canvas: NodeRef::default(),
            next_queue_canvas: NodeRef::default(),

            asset_cache: HashMap::new(),

            config,
        };
        renderer.populate_asset_cache();
        renderer
    }

    pub fn hold_piece_canvas(&self) -> Html {
        html! {
            <canvas ref={ self.hold_piece_canvas.clone() }
                    class="hold-piece-canvas"
                    width={ SIDE_BAR_WIDTH.to_string() }
                    height={ (LABEL_HEIGHT + PIECE_HEIGHT + SIDE_BAR_PADDING).to_string() }>
            </canvas>
        }
    }

    pub fn field_canvas(&self, field: &DefaultField<SrsTetromino>) -> Html {
        html! {
            <canvas ref={ self.field_canvas.clone() }
                    class="field-canvas"
                    // hide the hidden area of the board
                    style={ format!("margin-top: -{}px;", SQUARE_MUL * field.hidden()) }
                    width={ (SQUARE_MUL * field.width()).to_string() }
                    height={ (SQUARE_MUL * field.height()).to_string() }>
            </canvas>
        }
    }

    pub fn next_queue_canvas(&self, config: &ReadOnlyConfig) -> Html {
        html! {
            <canvas ref={ self.next_queue_canvas.clone() }
                    class="next-queue-canvas"
                    width={ SIDE_BAR_WIDTH.to_string() }
                    height={ (LABEL_HEIGHT + PIECE_HEIGHT * config.queue_len + SIDE_BAR_PADDING).to_string() }>
            </canvas>
        }
    }

    pub fn draw_hold_piece(&self, field: &DefaultField<SrsTetromino>) {
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

            if let Some(kind) = field.hold_piece() {
                self.draw_piece(kind, &context, SIDE_BAR_WIDTH / 2, LABEL_HEIGHT + PIECE_HEIGHT / 2)
            }
        }
    }

    pub fn draw_field(&self, field: &DefaultField<SrsTetromino>, first_render: bool) {
        // field width and height in squares
        let fw = field.width() as f64;
        let fh = field.height() as f64;

        // units in pixels
        let fw_px = SQUARE_MUL as f64 * fw;
        let fh_px = SQUARE_MUL as f64 * fh;
        let fhidden_end_px = (field.hidden() * SQUARE_MUL) as f64; // end of board hidden area

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
            for col in 1..field.width() {
                context.begin_path();
                context.move_to((col * SQUARE_MUL) as f64, fhidden_end_px);
                context.line_to((col * SQUARE_MUL) as f64, fh_px);
                context.stroke();
            }

            // horizontal grid lines (only for non-hidden board area)
            for row in field.hidden() + 1..field.height() {
                context.begin_path();
                context.move_to(0.0, (row * SQUARE_MUL) as f64);
                context.line_to(fw_px, (row * SQUARE_MUL) as f64);
                context.stroke();
            }

            context.set_global_alpha(self.config.shadow_opacity);
            let shadow_piece = field.shadow_piece();
            for Coords(row, col) in shadow_piece.coords() {
                self.draw_square(
                    &shadow_piece.kind(),
                    &context,
                    *row as usize * SQUARE_MUL,
                    *col as usize * SQUARE_MUL,
                );
            }

            context.set_global_alpha(1.0);
            for (row, line) in field.lines().iter().enumerate() {
                for (col, square) in line.squares().iter().enumerate() {
                    if let Square::Filled(kind) = square {
                        self.draw_square(kind, &context, row * SQUARE_MUL, col * SQUARE_MUL);
                    }
                }
            }
        }
    }

    pub fn draw_next_queue(
        &mut self,
        bag: &mut SingleBag<SrsTetromino>,
        config: &ReadOnlyConfig,
    ) {
        // total height of queue in pixels
        let nq_h_px = (LABEL_HEIGHT + PIECE_HEIGHT * config.queue_len + SIDE_BAR_PADDING) as f64;

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

            let queue = bag.peek().take(config.queue_len).cloned().collect::<Vec<_>>();

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

    fn draw_piece(&self, kind: SrsTetromino, context: &CanvasRenderingContext2d, x_offset: usize, y_offset: usize) {
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
    fn draw_square(&self, kind: &SrsTetromino, context: &CanvasRenderingContext2d, row: usize, col: usize) {
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

    fn populate_asset_cache(&mut self) {
        self.asset_cache = SrsTetromino::iter()
            .map(|kind| {
                let field_square_mul = SQUARE_MUL as u32;
                let image = HtmlImageElement::new_with_width_and_height(field_square_mul, field_square_mul).unwrap();
                let asset_src = format!("assets/skins/{}/{}.png", self.config.skin_name, kind.asset_name());
                image.set_src(&asset_src);
                (kind, image)
            })
            .collect();
    }

    pub fn update_config(&mut self, config: ReadOnlyConfig) {
        let skin_updated = self.config.skin_name != config.skin_name;
        self.config = config;
        if skin_updated {
            self.populate_asset_cache();
        }
    }
}
