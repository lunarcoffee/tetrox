use game::Game;
use sycamore::{component, generic_node::Html, prelude::Scope, view, view::View};
use tetrox::{tetromino::SrsTetromino, PieceKind};

mod game;

pub const SKIN_NAMES: &[&str] = &["tetrox", "gradient", "inset", "rounded", "solid"];

#[component]
fn AssetPreloader<'a, G: Html>(cx: &'a Scope<'a>) -> View<G> {
    let n_loaded = cx.create_signal(0);

    let assets = SrsTetromino::iter()
        .map(|k| k.asset_name().to_string())
        .chain(["grey".to_string()])
        .flat_map(|kind| {
            SKIN_NAMES.iter().map(move |skin| {
                let src = format!("assets/skins/{}/{}.png", skin, kind);
                view! { cx, img(class="loading-asset", src=src, on:load=|_| { n_loaded.set(*n_loaded.get() + 1) }) }
            })
        })
        .collect::<Vec<_>>();

    let n_total = assets.len();
    let assets_view = View::new_fragment(assets);

    if *n_loaded.get() < n_total {
        view! { cx, Game {} }
    } else {
        view! { cx,
            p(class="loading-text") { "Loading assets... (" (n_loaded.get()) "/" (n_total) ")" }
            (assets_view)
        }
    }
}

fn main() { sycamore::render(|cx| view! { cx, AssetPreloader {} }) }
