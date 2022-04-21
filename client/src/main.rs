#![feature(type_alias_impl_trait)]

use sycamore::{component, generic_node::Html, prelude::Scope, view, view::View, reactive};
use tetrox::{tetromino::SrsTetromino, PieceKind};
use crate::config::ConfigPanel;

mod board;
mod game;
mod config;
mod canvas;
mod util;
mod stats;

pub const SKIN_NAMES: &[&str] = &["tetrox", "gradient", "inset", "rounded", "solid"];

#[component]
fn AssetPreloader<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let n_loaded = reactive::create_signal(cx, 0);

    let assets = SrsTetromino::iter()
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
