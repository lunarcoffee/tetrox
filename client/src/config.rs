use std::fmt;
use std::{ops::Deref, str::FromStr};

use crate::board::Board;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement, InputEvent};
use yew::{html, Callback, Component, Context, Html};

#[derive(PartialEq, Clone)]
pub struct Config {
    // visual settings
    pub skin_name: String,
    pub field_zoom: f64,

    // field property settings
    pub field_width: usize,
    pub field_height: usize,
    pub field_hidden: usize,
    pub queue_len: usize,

    // gameplay
    pub gravity_delay: u32,
    pub lock_delay: u32,

    // handling
    pub delayed_auto_shift: u32,
    pub auto_repeat_rate: u32,
    pub soft_drop_rate: u32,

    // controls
    // TODO
}

impl Default for Config {
    fn default() -> Self {
        Config {
            skin_name: "tetrox".to_string(),
            field_zoom: 1.0,

            field_width: 10,
            field_height: 40,
            field_hidden: 20,
            queue_len: 5,

            gravity_delay: 1_000,
            lock_delay: 500,

            delayed_auto_shift: 120,
            auto_repeat_rate: 0,
            soft_drop_rate: 0,
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

    FieldWidth(usize),
    FieldHeight(usize),
    QueueLen(usize),

    GravityDelay(u32),
    LockDelay(u32),

    DelayedAutoShift(u32),
    AutoRepeatRate(u32),
    SoftDropRate(u32),
}

// config panel which wraps a `Board` component
pub struct ConfigPanelWrapper {
    config: Config,
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

    fn button_capture_input(label: &str) -> Html {
        html! {
            <div class="config-option">
                // <input type="button" value={ label } oninput={ callback }/>
            </div>
        }
    }
}

impl Component for ConfigPanelWrapper {
    type Message = ConfigMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        ConfigPanelWrapper {
            config: Config::default(), // TODO: retrieve from localstorage
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ConfigMessage::SkinName(skin_name) => self.config.skin_name = skin_name,
            ConfigMessage::FieldZoom(zoom) => self.config.field_zoom = zoom,

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
        }

        // TODO: update to localstorage
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
        let field_width_callback = make_update_callback!(HtmlInputElement, ConfigMessage::FieldWidth);
        let field_height_callback = make_update_callback!(HtmlInputElement, ConfigMessage::FieldHeight);
        let queue_len_callback = make_update_callback!(HtmlInputElement, ConfigMessage::QueueLen);
        let gravity_callback = make_update_callback!(HtmlInputElement, ConfigMessage::GravityDelay);
        let lock_delay_callback = make_update_callback!(HtmlInputElement, ConfigMessage::LockDelay);
        let das_callback = make_update_callback!(HtmlInputElement, ConfigMessage::DelayedAutoShift);
        let arr_callback = make_update_callback!(HtmlInputElement, ConfigMessage::AutoRepeatRate);
        let sdr_callback = make_update_callback!(HtmlInputElement, ConfigMessage::SoftDropRate);

        let config = &self.config;

        html! {
            <div class="content">
                <Board config={ ReadOnlyConfig(self.config.clone()) }/>
                <div class="config-panel">
                    { Self::section_heading("Visual") }
                    { Self::select_input("Block skin", crate::SKIN_NAMES, &config.skin_name, skin_name_callback) }
                    { Self::range_input("Field zoom", 0.25, 4.0, 0.05, config.field_zoom, field_zoom_callback) }

                    { Self::section_heading("Playfield") }
                    { Self::range_input("Field width", 4, 100, 1, config.field_width, field_width_callback) }
                    { Self::range_input("Field height", 3, 100, 1, config.field_height / 2, field_height_callback) }
                    { Self::range_input("Queue length", 0, 7, 1, config.queue_len, queue_len_callback) }

                    { Self::section_heading("Gameplay") }
                    { Self::range_input("Gravity delay", 10, 5_000, 1, config.gravity_delay, gravity_callback) }
                    { Self::range_input("Lock delay", 10, 3_000, 1, config.lock_delay, lock_delay_callback) }

                    { Self::section_heading("Handling") }
                    { Self::range_input("DAS", 0, 500, 1, config.delayed_auto_shift, das_callback) }
                    { Self::range_input("ARR", 0, 500, 1, config.auto_repeat_rate, arr_callback) }
                    { Self::range_input("SDR", 0, 500, 1, config.soft_drop_rate, sdr_callback) }

                    { Self::section_heading("Controls") }
                    { "todo lol" }
                </div>
            </div>
        }
    }
}
