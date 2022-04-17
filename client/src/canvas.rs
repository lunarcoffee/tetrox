use std::cell::RefCell;

use sycamore::{
    component,
    generic_node::{DomNode, Html},
    prelude::{create_effect, create_node_ref, use_context, NodeRef, Scope, Signal},
    view,
    view::View,
    Prop,
};
use tetrox::{
    field::{DefaultField, Square},
    Bag, Coords, PieceKind, SingleBag,
};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{board::AssetCache, config::Config, util};

pub const SQUARE_WIDTH: usize = 36; // the size of each square on the field

pub const LABEL_HEIGHT: usize = 30; // height of "hold" and "next" labels
pub const PIECE_HEIGHT: usize = SQUARE_WIDTH * 3; // height of hold/queue piece

pub const SIDE_BAR_WIDTH: usize = SQUARE_WIDTH * 5; // width of hold/queue panels
pub const SIDE_BAR_PADDING: usize = SQUARE_WIDTH / 6; // bottom padding of hold/queue panels

fn get_canvas_drawer<'a, P: PieceKind + 'static, G: Html>(
    config: &'a Config,
    canvas_ref: &NodeRef<G>,
    field: &'a DefaultField<P>,
    asset_cache: &'a AssetCache,
) -> Option<CanvasDrawer<'a, P>> {
    // get a `CanvasDrawer` for the given `canvas_ref`
    canvas_ref.try_get::<DomNode>().map(|node| {
        let canvas = node.unchecked_into::<HtmlCanvasElement>();
        let context = canvas.get_context("2d").unwrap().unwrap();
        let context = context.dyn_into::<CanvasRenderingContext2d>().unwrap();
        CanvasDrawer::new(config, asset_cache, field, context)
    })
}

#[component]
pub fn HoldPiece<'a, P: PieceKind + 'static, G: Html>(cx: Scope<'a>) -> View<G> {
    let hold_piece_ref = create_node_ref(cx);
    let view = view! { cx,
        canvas(
            ref=hold_piece_ref,
            class="hold-piece-canvas",
            width={ SIDE_BAR_WIDTH },
            height={ LABEL_HEIGHT + PIECE_HEIGHT + SIDE_BAR_PADDING },
        )
    };

    let config = use_context::<Signal<Config>>(cx);
    let field = use_context::<Signal<RefCell<DefaultField<P>>>>(cx);
    let asset_cache = use_context::<AssetCache>(cx);

    create_effect(cx, || {
        get_canvas_drawer(&config.get(), hold_piece_ref, &field.get().borrow(), asset_cache)
            .map(|c| c.draw_hold_piece());
    });

    view
}

#[component]
pub fn Field<'a, P: PieceKind + 'static, G: Html>(cx: Scope<'a>) -> View<G> {
    let field = use_context::<Signal<RefCell<DefaultField<P>>>>(cx);
    let field_ref = create_node_ref(cx);
    let view = view! { cx,
        canvas(
            ref=field_ref,
            class="field-canvas",
            width={ SQUARE_WIDTH * field.get().borrow().width() },
            height={ SQUARE_WIDTH * field.get().borrow().height() },
            style={ format!("margin-top: -{}px;", SQUARE_WIDTH * field.get().borrow().hidden()) },
        )
    };

    let config = use_context::<Signal<Config>>(cx);
    let asset_cache = use_context::<AssetCache>(cx);

    create_effect(cx, || {
        get_canvas_drawer(&config.get(), field_ref, &field.get().borrow(), asset_cache).map(|c| c.draw_field());
    });

    view
}

#[derive(Prop)]
pub struct NextQueueProps<'a, P: PieceKind> {
    bag: &'a Signal<RefCell<SingleBag<P>>>,
}

#[component]
pub fn NextQueue<'a, P: PieceKind + 'static, G: Html>(cx: Scope<'a>, props: NextQueueProps<'a, P>) -> View<G> {
    let config = use_context::<Signal<Config>>(cx);
    let next_queue_ref = create_node_ref(cx);
    let view = view! { cx,
        canvas(
            ref=next_queue_ref,
            class="next-queue-canvas",
            width={ SIDE_BAR_WIDTH },
            height={ LABEL_HEIGHT + PIECE_HEIGHT * config.get().queue_len + SIDE_BAR_PADDING },
        )
    };

    let field = use_context::<Signal<RefCell<DefaultField<P>>>>(cx);
    let asset_cache = use_context::<AssetCache>(cx);

    create_effect(cx, || {
        props.bag.track();
        get_canvas_drawer(&config.get(), next_queue_ref, &field.get().borrow(), asset_cache)
            .map(|c| c.draw_next_queue(props.bag));
    });

    view
}

pub struct CanvasDrawer<'a, P: PieceKind> {
    config: &'a Config,
    asset_cache: &'a AssetCache,
    field: &'a DefaultField<P>,
    context: CanvasRenderingContext2d,
}

impl<'a, P: PieceKind> CanvasDrawer<'a, P> {
    pub fn new(
        config: &'a Config,
        asset_cache: &'a AssetCache,
        field: &'a DefaultField<P>,
        context: CanvasRenderingContext2d,
    ) -> Self {
        CanvasDrawer {
            config,
            asset_cache,
            field,
            context,
        }
    }

    fn draw_hold_piece(&self) {
        let hp_h_px = (LABEL_HEIGHT + PIECE_HEIGHT + SIDE_BAR_PADDING) as f64;

        let ctx = &self.context;
        ctx.set_fill_style(&"black".into());
        ctx.clear_rect(0.0, 0.0, SIDE_BAR_WIDTH as f64, hp_h_px);

        // fill background
        ctx.set_stroke_style(&"black".into());
        ctx.set_global_alpha(0.6);
        ctx.fill_rect(0.0, 0.0, SIDE_BAR_WIDTH as f64, hp_h_px);

        // draw label
        ctx.set_fill_style(&"#bbb".into());
        ctx.set_global_alpha(1.0);
        ctx.set_font("18px 'IBM Plex Sans'");
        ctx.fill_text("hold", 8.0, 24.0).unwrap();

        if let Some(kind) = self.field.hold_piece() {
            self.draw_piece(kind, SIDE_BAR_WIDTH / 2, LABEL_HEIGHT + PIECE_HEIGHT / 2)
        }
    }

    fn draw_field(&self) {
        let field = self.field;

        // field width and height in squares
        let fw = field.width() as f64;
        let fh = field.height() as f64;

        // units in pixels
        let fw_px = SQUARE_WIDTH as f64 * fw;
        let fh_px = SQUARE_WIDTH as f64 * fh;
        let fhidden_end_px = (field.hidden() * SQUARE_WIDTH) as f64; // end of board hidden area

        let ctx = &self.context;
        ctx.set_fill_style(&"black".into());
        ctx.clear_rect(0.0, 0.0, fw_px, fh_px);

        // fill background
        ctx.set_global_alpha(0.6);
        ctx.fill_rect(0.0, fhidden_end_px, fw_px, fh_px);

        ctx.set_stroke_style(&"#555".into());
        ctx.set_global_alpha(0.3);

        // vertical grid lines
        for col in 1..field.width() {
            ctx.begin_path();
            ctx.move_to((col * SQUARE_WIDTH) as f64, fhidden_end_px);
            ctx.line_to((col * SQUARE_WIDTH) as f64, fh_px);
            ctx.stroke();
        }

        // horizontal grid lines (only for non-hidden board area)
        for row in field.hidden() + 1..field.height() {
            ctx.begin_path();
            ctx.move_to(0.0, (row * SQUARE_WIDTH) as f64);
            ctx.line_to(fw_px, (row * SQUARE_WIDTH) as f64);
            ctx.stroke();
        }

        ctx.set_global_alpha(self.config.shadow_opacity);
        ctx.set_global_alpha(0.3);
        let shadow_piece = field.shadow_piece();
        let topped_out = field.topped_out() && self.config.topping_out_enabled;

        if !topped_out {
            for Coords(row, col) in shadow_piece.coords() {
                let kind = shadow_piece.kind();
                let asset = kind.asset_name();
                self.draw_square(asset, *row as usize * SQUARE_WIDTH, *col as usize * SQUARE_WIDTH);
            }
        }

        ctx.set_global_alpha(1.0);
        for (row, line) in field.lines().iter().enumerate() {
            for (col, square) in line.squares().iter().enumerate() {
                if let Square::Filled(kind) = square {
                    let asset = if topped_out { "grey" } else { kind.asset_name() };
                    self.draw_square(asset, row * SQUARE_WIDTH, col * SQUARE_WIDTH);
                }
            }
        }
    }

    fn draw_next_queue(&self, bag: &Signal<RefCell<SingleBag<P>>>) {
        let queue_len = self.config.queue_len;

        // total height of queue in pixels
        let nq_h_px = (LABEL_HEIGHT + PIECE_HEIGHT * self.config.queue_len + SIDE_BAR_PADDING) as f64;

        let ctx = &self.context;
        ctx.set_fill_style(&"black".into());
        ctx.clear_rect(0.0, 0.0, SIDE_BAR_WIDTH as f64, nq_h_px);

        // fill background
        ctx.set_stroke_style(&"black".into());
        ctx.set_global_alpha(0.6);
        ctx.fill_rect(0.0, 0.0, SIDE_BAR_WIDTH as f64, nq_h_px);

        // draw label
        ctx.set_fill_style(&"#bbb".into());
        ctx.set_global_alpha(1.0);
        ctx.set_font("18px 'IBM Plex Sans'");
        ctx.fill_text("next", 8.0, 24.0).unwrap();

        // TODO: does this refcell usage work lol
        let queue = util::with_signal_mut(bag, |bag| bag.peek().take(queue_len).cloned().collect::<Vec<_>>());
        for (nth, kind) in queue.iter().enumerate() {
            self.draw_piece(
                *kind,
                SIDE_BAR_WIDTH / 2,
                LABEL_HEIGHT + PIECE_HEIGHT * (nth + 1) - PIECE_HEIGHT / 2,
            )
        }
    }

    fn draw_piece(&self, kind: P, x_offset: usize, y_offset: usize) {
        let base_coords = kind
            .spawn_offsets()
            .into_iter()
            .map(|Coords(row, col)| Coords(row * SQUARE_WIDTH as i32, col * SQUARE_WIDTH as i32))
            .collect();

        let offset = Coords(y_offset as i32, x_offset as i32);
        let final_coords = Self::center_coords_around_origin(base_coords)
            .into_iter()
            .map(|c| c + offset);

        for Coords(row, col) in final_coords {
            self.draw_square(kind.asset_name(), row as usize, col as usize);
        }
    }

    // draw a square at the given coords on a canvas
    fn draw_square(&self, asset_name: &str, row: usize, col: usize) {
        let asset_name = format!("assets/skins/tetrox/{}.png", asset_name);
        let asset = &self.asset_cache.get(&asset_name).unwrap();

        self.context
            .draw_image_with_html_image_element_and_dw_and_dh(
                asset,
                col as f64,
                row as f64,
                SQUARE_WIDTH as f64,
                SQUARE_WIDTH as f64,
            )
            .unwrap();
    }

    fn center_coords_around_origin(coords: Vec<Coords>) -> Vec<Coords> {
        let min_col = coords.iter().min_by_key(|Coords(_, col)| col).unwrap().1;
        let max_col = coords.iter().max_by_key(|Coords(_, col)| col).unwrap().1;
        let min_row = coords.iter().min_by_key(|Coords(row, _)| row).unwrap().0;
        let max_row = coords.iter().max_by_key(|Coords(row, _)| row).unwrap().0;

        let offset = Coords((max_row + min_row) / 2, (max_col + min_col) / 2);
        coords
            .into_iter()
            // (0, 0) is not the center since images are drawn from the top-left corner
            // the actual center is half a `SQUARE_WIDTH` away in both directions
            .map(|c| c - offset - Coords(SQUARE_WIDTH as i32 / 2, SQUARE_WIDTH as i32 / 2))
            .collect()
    }
}
