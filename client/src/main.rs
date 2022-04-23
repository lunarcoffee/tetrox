#![feature(type_alias_impl_trait)]

use crate::config::ConfigPanel;
use sycamore::{component, generic_node::Html, prelude::Scope, reactive, view, view::View};
use tetrox::pieces::{tetromino::TetrominoSrs, PieceKindTrait};

mod board;
mod canvas;
mod config;
mod game;
mod stats;
mod util;
mod timer;
mod goal;

pub const SKIN_NAMES: &[&str] = &["tetrox", "gradient", "inset", "rounded", "tetrio", "solid"];

#[component]
fn AssetPreloader<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let n_loaded = reactive::create_signal(cx, 0);

    let assets = TetrominoSrs::iter()
        .map(|k| k.asset_name().to_string())
        .chain(["grey".to_string()])
        .flat_map(|kind| {
            SKIN_NAMES.iter().map(move |skin| {
                let src = format!("assets/skins/{}/{}.png", skin, kind);
                view! { cx, img(class="loading-asset", src=src, on:load=|_| { n_loaded.set(*n_loaded.get() + 1) }) }
            })
        })
        // collect to use iterator and start image loading
        .collect::<Vec<View<G>>>();

    let n_total = assets.len();
    view! { cx,
        div(class="bg-gradient")
        (if *n_loaded.get() == n_total { // show the game once all assets have loaded
            view! { cx, ConfigPanel {} }
        } else {
            view! { cx, p(class="loading-text") { "Loading assets... (" (n_loaded.get()) "/" (n_total) ")" } }
        })
    }
}

fn main() { sycamore::render(|cx| view! { cx, AssetPreloader {} }) }
