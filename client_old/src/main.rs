#![feature(trait_alias)]

use config::ConfigPanelWrapper;
use tetrox::{tetromino::SrsTetromino, PieceKind};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::HtmlImageElement;
use yew::{html, Component, Context, Properties};

mod animation;
mod board;
mod canvas;
mod config;
mod game_stats;
mod input;
mod inputs;

// a single asset has been loaded
struct AssetLoaded;

// the total number of assets to be loaded (used to check load completion)
#[derive(Clone, PartialEq, Properties)]
struct AssetPreloaderProps {
    n_assets: usize,
}

struct AssetPreloader {
    n_loaded: usize,

    loaded_assets: Vec<HtmlImageElement>, // storing these so they don't get uncached after being dropped
    loaded_callback_closures: Vec<Closure<dyn Fn()>>, // storing these so they aren't dropped before being called
}

// maybe dynamically determine these?
pub const SKIN_NAMES: &[&str] = &["tetrox", "gradient", "inset", "rounded", "solid"];

impl AssetPreloader {
    fn register_asset_load_callbacks(&mut self, ctx: &Context<Self>) {
        let kinds = SrsTetromino::iter().map(|k| k.asset_name().to_string()).chain(["grey".to_string()]);
        for kind in kinds {
            for skin_name in SKIN_NAMES {
                let image = HtmlImageElement::new().unwrap();
                let asset_src = format!("assets/skins/{}/{}.png", skin_name, kind);
                image.set_src(&asset_src);

                let link = ctx.link().clone();
                let loaded_callback = move || link.send_message(AssetLoaded);
                let loaded_closure = Closure::wrap(Box::new(loaded_callback) as Box<dyn Fn()>);
                image.set_onload(Some(loaded_closure.as_ref().unchecked_ref()));

                self.loaded_assets.push(image);
                self.loaded_callback_closures.push(loaded_closure);
            }
        }
    }
}

impl Component for AssetPreloader {
    type Message = AssetLoaded;
    type Properties = AssetPreloaderProps;

    fn create(_ctx: &Context<Self>) -> Self {
        AssetPreloader {
            n_loaded: 0,

            loaded_assets: vec![],
            loaded_callback_closures: vec![],
        }
    }

    fn update(&mut self, ctx: &Context<Self>, _msg: Self::Message) -> bool {
        self.n_loaded += 1;
        self.n_loaded == ctx.props().n_assets
    }

    fn view(&self, ctx: &Context<Self>) -> yew::Html {
        if self.n_loaded == ctx.props().n_assets {
            html! { <ConfigPanelWrapper/> }
        } else {
            html! { <p class="loading-text">{ "Loading..." }</p> }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.register_asset_load_callbacks(ctx);
        }
    }
}

fn main() {
    let n_assets = (SrsTetromino::iter().count() + 1) * SKIN_NAMES.len();
    yew::start_app_with_props::<AssetPreloader>(AssetPreloaderProps { n_assets });
}
