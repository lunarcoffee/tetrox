#![feature(stmt_expr_attributes)]

use tetrox::{
    field::{DefaultField, Square},
    tetromino::{SevenBag, Tetromino},
    PieceKind,
};
use yew::{html, html_nested, Component, Context, KeyboardEvent, Html};

enum BoardMessage {
    KeyPressed(KeyboardEvent),
}

struct BoardModel {
    bag: SevenBag,
    field: DefaultField<Tetromino>,
}

impl Component for BoardModel {
    type Message = BoardMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut bag = SevenBag::new();
        let field = DefaultField::new(10, 40, 20, &mut bag);
        BoardModel { bag, field }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            BoardMessage::KeyPressed(e) => match &e.key()[..] {
                "ArrowLeft" => self.field.try_shift(0, -1),
                "ArrowRight" => self.field.try_shift(0, 1),
                "ArrowDown" => self.field.try_shift(1, 0),
                "ArrowUp" => self.field.try_shift(-1, 0),
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
            <div class="field" tabindex="0" onkeydown={ key_pressed_callback }>{
                for self.field.lines().iter().map(|line|
                    html_nested! {
                        <div class="field-line">{
                            for line.squares().iter().map(|square|
                                match square {
                                    Square::Empty => html_nested! {
                                        <div class="field-square-empty field-square"></div>
                                    },
                                    Square::Filled(kind) => {
                                        let file_name = format!("assets/{}.png", kind.asset_name());
                                        html_nested! {
                                            <div class="field-square">
                                                <img class="field-square" alt="hello" src={ file_name }/>
                                            </div>
                                        }
                                    },
                                }
                            )
                        }</div>
                    }
                )
            }</div>
        }
    }
}

fn main() {
    yew::start_app::<BoardModel>();
}
