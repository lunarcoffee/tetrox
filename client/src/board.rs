use std::cell::RefCell;
use std::collections::HashMap;

use crate::canvas::{self, FieldCanvas, HoldPieceCanvas, NextQueueCanvas};
use sycamore::{component, generic_node::Html, prelude::Scope, view, view::View};
use tetrox::{field::DefaultField, tetromino::SrsTetromino};
use tetrox::{PieceKind, SingleBag};
use web_sys::HtmlImageElement;

pub type AssetCache = HashMap<String, HtmlImageElement>;

fn make_asset_cache() -> AssetCache {
    SrsTetromino::iter() // TODO: better way of getting all assets
        .map(|k| k.asset_name().to_string())
        .chain(["grey".to_string()])
        .map(|asset_name| {
            let field_square_mul = canvas::SQUARE_WIDTH as u32;
            let image = HtmlImageElement::new_with_width_and_height(field_square_mul, field_square_mul).unwrap();

            let asset_src = format!("assets/skins/{}/{}.png", "tetrox", asset_name);
            image.set_src(&asset_src);

            (asset_src.to_string(), image)
        })
        .collect()
}

#[component]
pub fn Board<'a, P: PieceKind + 'static, G: Html>(cx: &'a Scope<'a>) -> View<G> {
    // (config.field_width, config.field_height, config.field_hidden, &mut bag)
    let mut bag = SingleBag::<P>::new();
    let field = cx.create_signal(DefaultField::new(10, 40, 20, &mut bag)); // TODO: CONFIG
    cx.provide_context_ref(field);

    let bag = cx.create_signal(RefCell::new(bag));

    let asset_cache = make_asset_cache();
    cx.provide_context(asset_cache); // provide to all asset consumers

    view! { cx,
        div(class="game", tabindex="0") {
            div(class="field-panel") { div(class="hold-piece") { HoldPieceCanvas::<P, G> {} } }
            div(class="field") { FieldCanvas::<P, G> {} }
            div(class="next-queue") { NextQueueCanvas { bag: bag } }
        }
    }
}
