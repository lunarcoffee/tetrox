use board::Board;
use strum::IntoEnumIterator;
use tetrox::{tetromino::SrsTetromino, PieceKind};
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
    loaded_callback_closures: Vec<Closure<dyn Fn()>>, // storing these so they aren't dropped before being called
}

impl AssetPreloader {
    fn register_asset_load_callbacks(&mut self, link: &Scope<Self>) {
        for kind in SrsTetromino::iter() {
            let image = HtmlImageElement::new().unwrap();
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
                    html! { <Board width=10 height=40 hidden=20 queue_len=5/> }
                } else {
                    html! { <p class="loading-text">{ "Loading..." }</p> }
                }
            }</div>
        }
    }

    fn rendered(&mut self, ctx: &yew::Context<Self>, first_render: bool) {
        if first_render {
            self.register_asset_load_callbacks(ctx.link());
        }
    }
}

fn main() {
    let n_assets = SrsTetromino::iter().count();
    yew::start_app_with_props::<AssetPreloader>(AssetPreloaderProps { n_assets });
}
