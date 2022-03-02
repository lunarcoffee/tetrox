#![feature(stmt_expr_attributes)]

use tetrox::{
    field::{DefaultField, Square},
    tetromino::{SevenBag, Tetromino},
    Bag,
};
use yew::{html, html_nested, Component, Context, KeyboardEvent};

enum BoardMessage {
    Move(KeyboardEvent),
}

struct BoardModel {
    bag: SevenBag,
    field: DefaultField<Tetromino>,
}

impl Component for BoardModel {
    type Message = BoardMessage;
    type Properties = ();

    fn create(ctx: &yew::Context<Self>) -> Self {
        let mut bag = SevenBag::new();
        let field = DefaultField::new(10, 40, 20, &mut bag);
        BoardModel { bag, field }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            BoardMessage::Move(e) => match &e.key()[..] {
                "ArrowLeft" => self.field.try_shift(0, -1),
                "ArrowRight" => self.field.try_shift(0, 1),
                "ArrowDown" => self.field.try_shift(1, 0),
                " " => self.field.try_spawn_no_erase(&mut self.bag),
                _ => return false,
            },
        };
        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let link = ctx.link();
        html! {
            <div>
                <p tabindex="0" onkeydown={ link.callback(|e: KeyboardEvent| BoardMessage::Move(e)) }>{
                    for self.field.lines().iter().map(|line|
                        html_nested! {
                            <p style="line-height: 0;font-family: monospace;">{
                                line.squares()
                                    .iter()
                                    .map(|s| match s {
                                        Square::Empty => "_",
                                        Square::Filled(_) => "#",
                                    }).collect::<Vec<_>>()
                                    .join("")
                            }</p>
                        }
                    )
                }</p>
            </div>
        }
    }
}

fn main() {
    let mut bag = SevenBag::new();
    let mut playfield = DefaultField::<Tetromino>::new(10, 40, 20, &mut bag);
    playfield.try_spawn(&mut bag);

    // playfield.try_shift(0, 5);

    // for line in playfield.lines() {
    //     let squares = line.squares();
    //     println!(
    //         "{}",
    //         squares.iter().map(|s| match s {
    //             Square::Empty => "_",
    //             Square::Filled(_) => "#",
    //         }).collect::<Vec<_>>().join("")
    //     );
    // }

    // println!("client");
    yew::start_app::<BoardModel>();
}
