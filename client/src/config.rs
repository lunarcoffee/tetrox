use crate::game::Game;

use bimap::BiMap;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use sycamore::{
    component,
    generic_node::Html,
    prelude::{create_signal, provide_context_ref, Scope},
    view,
    view::View,
};

#[component]
fn SectionHeading<'a, G: Html>(cx: Scope<'a>, section: &'static str) -> View<G> {
    view! { cx, p(class="config-heading") { (section) } }
}

#[component]
pub fn ConfigPanel<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let config = create_signal(cx, Config::default());
    provide_context_ref(cx, config);

    view! { cx,
        div(class="content") {
            Game {}
            div(class="config-panel") {
                SectionHeading("Gameplay")
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter)]
pub enum Input {
    Left,
    Right,
    SoftDrop,
    HardDrop,
    RotateCw,
    RotateCcw,
    Rotate180,
    SwapHoldPiece,
    Reset,
    ShowHideUi,
}

pub type Keybind = String;

// TODO: std::any::Any?
#[derive(Serialize, Deserialize)]
pub struct Config {
    // visual settings
    pub skin_name: String,
    pub field_zoom: f64,
    pub vertical_offset: i32,
    pub shadow_opacity: f64,

    // field property settings
    pub field_width: usize,
    pub field_height: usize,
    pub field_hidden: usize,
    pub queue_len: usize,

    // gameplay
    pub gravity_delay: u32,
    pub lock_delay: u32,
    pub move_limit: usize,
    pub topping_out_enabled: bool,
    pub auto_lock_enabled: bool,
    pub gravity_enabled: bool,
    pub move_limit_enabled: bool,

    // controls
    pub inputs: BiMap<Input, Keybind>,

    // handling
    pub delayed_auto_shift: u32,
    pub auto_repeat_rate: u32,
    pub soft_drop_rate: u32,
}

impl Default for Config {
    fn default() -> Self {
        // guideline controls (minus double binds)
        let inputs = [
            (Input::Left, "ArrowLeft"),
            (Input::Right, "ArrowRight"),
            (Input::SoftDrop, "ArrowDown"),
            (Input::HardDrop, " "),
            (Input::RotateCw, "x"),
            (Input::RotateCcw, "z"),
            (Input::Rotate180, "Shift"),
            (Input::SwapHoldPiece, "c"),
            (Input::Reset, "`"),
            (Input::ShowHideUi, "F9"),
        ];

        Config {
            skin_name: "tetrox".to_string(),
            field_zoom: 1.0,
            vertical_offset: 170,
            shadow_opacity: 0.3,

            field_width: 10,
            field_height: 40,
            field_hidden: 20,
            queue_len: 5,

            gravity_delay: 1_000,
            lock_delay: 500,
            move_limit: 30,
            topping_out_enabled: true,
            auto_lock_enabled: true,
            gravity_enabled: true,
            move_limit_enabled: true,

            delayed_auto_shift: 280,
            auto_repeat_rate: 50,
            soft_drop_rate: 30,

            inputs: inputs.into_iter().map(|(i, k)| (i, k.to_string())).collect(),
        }
    }
}
