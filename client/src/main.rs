#![feature(stmt_expr_attributes)]

use board::Board;
use strum::IntoEnumIterator;
use tetrox::{tetromino::Tetromino, PieceKind};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::HtmlImageElement;
use yew::{html, html::Scope, Component, Properties};

mod board;
mod input;

const SKIN_NAME: &str = "tetrox";

// a single asset has been loaded
struct AssetLoaded;

// the total number of assets to be loaded (used to check load completion)
#[derive(Clone, PartialEq, Properties)]
struct AssetPreloaderProps {
    n_assets: usize,
}

struct AssetPreloader {
    n_loaded: usize,

    // storing these so they aren't dropped before they're called
    loaded_callback_closures: Vec<Closure<dyn Fn()>>,
}

impl AssetPreloader {
    fn populate_asset_cache(&mut self, link: &Scope<Self>) {
        for kind in Tetromino::iter() {
            let field_square_mul = board::SQUARE_MUL as u32;
            let image = HtmlImageElement::new_with_width_and_height(field_square_mul, field_square_mul).unwrap();

            let asset_src = format!("assets/skins/{}/{}.png", SKIN_NAME, kind.asset_name());
            image.set_src(&asset_src);

            let link = link.clone();
            let loaded_callback = move || link.send_message(AssetLoaded);
            let loaded_closure = Closure::wrap(Box::new(loaded_callback) as Box<dyn Fn()>);
            image.set_onload(Some(loaded_closure.as_ref().unchecked_ref()));
            self.loaded_callback_closures.push(loaded_closure);
        }
    }
}

impl Component for AssetPreloader {
    type Message = AssetLoaded;
    type Properties = AssetPreloaderProps;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        AssetPreloader {
            n_loaded: 0,
            loaded_callback_closures: vec![],
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, _msg: Self::Message) -> bool {
        self.n_loaded += 1;
        self.n_loaded == ctx.props().n_assets
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        html! {
            <div>{
                if self.n_loaded == ctx.props().n_assets {
                    html! { <Board /> }
                } else {
                    html! { <p class="loading-text">{ "Loading..." }</p> }
                }
            }</div>
        }
    }

    fn rendered(&mut self, ctx: &yew::Context<Self>, first_render: bool) {
        if first_render {
            self.populate_asset_cache(ctx.link());
        }
    }
}

fn main() {
    let props = AssetPreloaderProps {
        n_assets: Tetromino::iter().count(),
    };
    yew::start_app_with_props::<AssetPreloader>(props);
}
