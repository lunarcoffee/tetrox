#![feature(stmt_expr_attributes)]

use tetrox::{
    field::{DefaultField, Square},
    tetromino::{ExtendedSrsKickTable, SevenBag, SrsKickTable, Tetromino},
    PieceKind,
};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
use yew::{html, Component, Context, Html, KeyboardEvent, NodeRef};

enum BoardMessage {
    KeyPressed(KeyboardEvent),
}

struct BoardModel {
    bag: SevenBag,
    field: DefaultField<Tetromino>,
    canvas: NodeRef,
}

impl Component for BoardModel {
    type Message = BoardMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut bag = SevenBag::new();
        let field = DefaultField::new(10, 40, 20, &mut bag);

        BoardModel {
            bag,
            field,
            canvas: NodeRef::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            BoardMessage::KeyPressed(e) => match &e.key()[..] {
                "ArrowLeft" => self.field.try_shift(0, -1),
                "ArrowRight" => self.field.try_shift(0, 1),
                "ArrowDown" => self.field.try_shift(1, 0),
                "ArrowUp" => self.field.try_rotate_cw(&SrsKickTable),
                "s" => self.field.try_rotate_ccw(&SrsKickTable),
                "a" => self.field.try_rotate_180(&ExtendedSrsKickTable),
                " " => self.field.try_spawn_no_erase(&mut self.bag),
                _ => return false,
            },
        };
        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let link = ctx.link();
        let key_pressed_callback = link.callback(|e| BoardMessage::KeyPressed(e));

        html! {
            <div class="field">
                <canvas ref={ self.canvas.clone() }
                        class="field-canvas"
                        tabindex="0"
                        onkeydown={ key_pressed_callback }
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

            context.clear_rect(0., 0., 32. * 10., 32. * 40.);

            for (y_offset, line) in self.field.lines().iter().enumerate() {
                for (x_offset, square) in line.squares().iter().enumerate() {
                    match square {
                        Square::Filled(kind) => {
                            let image_elem = HtmlImageElement::new_with_width_and_height(32, 32).unwrap();
                            let asset_src = format!("assets/{}.png", kind.asset_name());
                            image_elem.set_src(&asset_src);
                            context
                                .draw_image_with_html_image_element_and_dw_and_dh(
                                    &image_elem,
                                    32. * x_offset as f64,
                                    32. * y_offset as f64,
                                    32.,
                                    32.,
                                )
                                .unwrap();
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn main() { yew::start_app::<BoardModel>(); }
