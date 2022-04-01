use std::fmt;
use std::ops::Deref;

use crate::board::{Board, Keybind};
use bimap::BiMap;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement, InputEvent, KeyboardEvent, Storage};
use yew::{html, Callback, Component, Context, Html};

#[derive(Copy, Clone, PartialEq, Eq, Hash, EnumIter, Serialize, Deserialize)]
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
}

#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub struct Config {
    // visual settings
    pub skin_name: String,
    pub field_zoom: f64,
    pub vertical_offset: i32,

    // field property settings
    pub field_width: usize,
    pub field_height: usize,
    pub field_hidden: usize,
    pub queue_len: usize,

    // gameplay
    pub gravity_delay: u32,
    pub lock_delay: u32,

    // controls
    pub inputs: BiMap<Input, Keybind>,

    // handling
    pub delayed_auto_shift: u32,
    pub auto_repeat_rate: u32,
    pub soft_drop_rate: u32,
}

const CONFIG_LOCAL_STORAGE_KEY: &str = "config";

impl Config {
    fn from_local_storage(storage: Storage) -> Option<Self> {
        let json = storage.get_item(CONFIG_LOCAL_STORAGE_KEY).ok()??;
        serde_json::from_str(&json).ok()
    }
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
        ]
        .into_iter()
        .map(|(i, k)| (i, k.to_string()))
        .collect();

        Config {
            skin_name: "tetrox".to_string(),
            field_zoom: 1.0,
            vertical_offset: 130,

            field_width: 10,
            field_height: 40,
            field_hidden: 20,
            queue_len: 5,

            gravity_delay: 1_000,
            lock_delay: 500,

            delayed_auto_shift: 280,
            auto_repeat_rate: 50,
            soft_drop_rate: 30,

            inputs,
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct ReadOnlyConfig(Config);

impl Deref for ReadOnlyConfig {
    type Target = Config;

    fn deref(&self) -> &Self::Target { &self.0 }
}

pub enum ConfigMessage {
    SkinName(String),
    FieldZoom(f64),
    VerticalOffset(i32),

    FieldWidth(usize),
    FieldHeight(usize),
    QueueLen(usize),

    GravityDelay(u32),
    LockDelay(u32),

    DelayedAutoShift(u32),
    AutoRepeatRate(u32),
    SoftDropRate(u32),

    StartRebindInput(Input),
    CancelRebindInput,
    RebindInput(String),

    ResetToDefault,
}

// config panel which wraps a `Board` component
pub struct ConfigPanelWrapper {
    config: Config,
    capturing_input: Option<Input>, // `Some` when capturing a new keybind
}

impl ConfigPanelWrapper {
    fn section_heading(name: &str) -> Html {
        html! { <p class="config-heading">{ name }</p> }
    }

    fn range_input<T>(label: &str, min: T, max: T, step: T, value: T, callback: Callback<InputEvent>) -> Html
    where
        T: fmt::Display,
    {
        html! {
            <div class="config-option">
                { Self::input_label(label, &value) }
                <input type="range" min={ min.to_string() } max={ max.to_string() } step={ step.to_string() }
                       value={ value.to_string() }
                       oninput={ callback }/>
            </div>
        }
    }

    fn select_input(label: &str, items: &[&'static str], selected: &str, callback: Callback<InputEvent>) -> Html {
        html! {
            <div class="config-option">
                { Self::input_label(label, &selected) }
                <select oninput={ callback }>{
                    for items.iter().map(|i| {
                        html! { <option value={ *i } selected={ selected == *i }>{ i }</option> }
                    })
                }</select>
            </div>
        }
    }

    fn input_label(label: &str, value: &impl fmt::Display) -> Html {
        html! { <p class="config-option-label">{ format!("{} ({}):", label, value) }</p> }
    }

    // button that reads a new keybind for the given input
    fn button_capture_input(&self, ctx: &Context<Self>, label: &str, input: Input) -> Html {
        let keybind = self
            .config
            .inputs
            .get_by_left(&input)
            .map(|k| match self.capturing_input.as_ref() {
                Some(rebinding_input) if input == *rebinding_input => "<press a key>",
                _ => match k.as_str() {
                    " " => "Space",
                    _ if k.starts_with("Arrow") => &k[5..],
                    _ => k.as_str(),
                },
            })
            .unwrap_or("<unset>");

        let label = format!("{} ({})", label, keybind);
        let start_callback = ctx.link().callback(move |_| ConfigMessage::StartRebindInput(input));
        let rebind_callback = ctx.link().callback(move |e: KeyboardEvent| {
            let key = e.key();
            if key.starts_with("Esc") {
                ConfigMessage::CancelRebindInput
            } else {
                // `BiMap` automatically overrides duplicate binds
                ConfigMessage::RebindInput(e.key())
            }
        });

        html! {
            <div class="config-option">
                <input type="button" value={ label } onclick={ start_callback } onkeydown={ rebind_callback }/>
            </div>
        }
    }

    fn store_config(&self) {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        let json = serde_json::to_string(&self.config).expect("amongus");
        storage.set_item(CONFIG_LOCAL_STORAGE_KEY, &json).expect("sussus");
    }
}

impl Component for ConfigPanelWrapper {
    type Message = ConfigMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();

        ConfigPanelWrapper {
            config: Config::from_local_storage(storage).unwrap_or_else(|| Config::default()),
            capturing_input: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        // regular value updates
        match msg {
            ConfigMessage::SkinName(ref skin_name) => self.config.skin_name = skin_name.to_string(),
            ConfigMessage::FieldZoom(zoom) => self.config.field_zoom = zoom,
            ConfigMessage::VerticalOffset(offset) => self.config.vertical_offset = offset,

            ConfigMessage::FieldWidth(width) => self.config.field_width = width,
            ConfigMessage::FieldHeight(height) => {
                self.config.field_height = height * 2;
                self.config.field_hidden = height;
            }
            ConfigMessage::QueueLen(queue_len) => self.config.queue_len = queue_len,

            ConfigMessage::GravityDelay(gravity) => self.config.gravity_delay = gravity,
            ConfigMessage::LockDelay(lock_delay) => self.config.lock_delay = lock_delay,

            ConfigMessage::DelayedAutoShift(das) => self.config.delayed_auto_shift = das,
            ConfigMessage::AutoRepeatRate(arr) => self.config.auto_repeat_rate = arr,
            ConfigMessage::SoftDropRate(sdr) => self.config.soft_drop_rate = sdr,

            ConfigMessage::ResetToDefault => self.config = Config::default(),
            _ => {}
        }

        // input keybind updates
        match msg {
            ConfigMessage::StartRebindInput(input) => self.capturing_input = Some(input),
            ConfigMessage::CancelRebindInput => self.capturing_input = None,
            ConfigMessage::RebindInput(keybind) => {
                if let Some(input) = self.capturing_input {
                    self.config.inputs.insert(input, keybind);
                    self.capturing_input = None;
                }
            }
            _ => {}
        }

        self.store_config();
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        macro_rules! make_update_callback {
            ($type:ty, $msg:expr) => {
                ctx.link().batch_callback(move |e: InputEvent| {
                    let input = e.target().unwrap().dyn_into::<$type>();
                    input.unwrap().value().parse().ok().map(|v| $msg(v))
                })
            };
        }

        let skin_name_callback = make_update_callback!(HtmlSelectElement, ConfigMessage::SkinName);
        let field_zoom_callback = make_update_callback!(HtmlInputElement, ConfigMessage::FieldZoom);
        let offset_callback = make_update_callback!(HtmlInputElement, ConfigMessage::VerticalOffset);

        let field_width_callback = make_update_callback!(HtmlInputElement, ConfigMessage::FieldWidth);
        let field_height_callback = make_update_callback!(HtmlInputElement, ConfigMessage::FieldHeight);
        let queue_len_callback = make_update_callback!(HtmlInputElement, ConfigMessage::QueueLen);

        let gravity_callback = make_update_callback!(HtmlInputElement, ConfigMessage::GravityDelay);
        let lock_delay_callback = make_update_callback!(HtmlInputElement, ConfigMessage::LockDelay);

        let das_callback = make_update_callback!(HtmlInputElement, ConfigMessage::DelayedAutoShift);
        let arr_callback = make_update_callback!(HtmlInputElement, ConfigMessage::AutoRepeatRate);
        let sdr_callback = make_update_callback!(HtmlInputElement, ConfigMessage::SoftDropRate);

        let reset_defaults_callback = ctx.link().callback(|_| ConfigMessage::ResetToDefault);

        let config = &self.config;

        html! {
            <div class="content">
                <Board config={ ReadOnlyConfig(self.config.clone()) }/>
                <div class="config-panel">
                    { Self::section_heading("Visual") }
                    { Self::select_input("Block skin", crate::SKIN_NAMES, &config.skin_name, skin_name_callback) }
                    { Self::range_input("Field zoom", 0.1, 4.0, 0.05, config.field_zoom, field_zoom_callback) }
                    { Self::range_input("Vertical offset", -2_000, 2_000, 10, config.vertical_offset, offset_callback) }

                    { Self::section_heading("Playfield") }
                    { Self::range_input("Field width", 4, 100, 1, config.field_width, field_width_callback) }
                    { Self::range_input("Field height", 3, 100, 1, config.field_height / 2, field_height_callback) }
                    { Self::range_input("Queue length", 0, 7, 1, config.queue_len, queue_len_callback) }

                    { Self::section_heading("Gameplay") }
                    { Self::range_input("Gravity delay", 10, 5_000, 5, config.gravity_delay, gravity_callback) }
                    { Self::range_input("Lock delay", 10, 3_000, 5, config.lock_delay, lock_delay_callback) }

                    { Self::section_heading("Keybinds") }
                    <div class="config-button-box">
                        { self.button_capture_input(ctx, "Left", Input::Left) }
                        { self.button_capture_input(ctx, "Right", Input::Right) }
                        { self.button_capture_input(ctx, "Soft drop", Input::SoftDrop) }
                        { self.button_capture_input(ctx, "Hard drop", Input::HardDrop) }
                        { self.button_capture_input(ctx, "Rotate CW", Input::RotateCw) }
                        { self.button_capture_input(ctx, "Rotate CCW", Input::RotateCcw) }
                        { self.button_capture_input(ctx, "Rotate 180", Input::Rotate180) }
                        { self.button_capture_input(ctx, "Swap hold", Input::SwapHoldPiece) }
                        { self.button_capture_input(ctx, "Reset", Input::Reset) }
                    </div>

                    { Self::section_heading("Handling") }
                    { Self::range_input("DAS", 0, 500, 1, config.delayed_auto_shift, das_callback) }
                    { Self::range_input("ARR", 0, 500, 1, config.auto_repeat_rate, arr_callback) }
                    { Self::range_input("SDR", 0, 500, 1, config.soft_drop_rate, sdr_callback) }

                    { Self::section_heading("Misc") }
                    <p class="config-option-label">{ "tetrox by lunarcoffee" }</p>
                    <a class="config-option-label link"  href="https://github.com/lunarcoffee/tetrox" target="_blank">
                        { "github" }
                    </a>
                    <div class="config-option" style="margin: 10px 0 6px 0;">
                        <input class="config-reset-button" type="button" value={ "Reset all to default" } 
                               onclick={ reset_defaults_callback }/>
                    </div>
                </div>
            </div>
        }
    }
}
