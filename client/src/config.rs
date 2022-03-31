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
    pub field_zoom: f64, // TODO

    // field property settings
    pub field_width: usize,
    pub field_height: usize,
    pub field_hidden: usize,
    pub queue_len: usize,

    // controls
    // TODO

    // handling
    pub delayed_auto_shift: u32,
    pub auto_repeat_rate: u32,
    pub soft_drop_rate: u32,
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
                self.config.field_height = height;
                self.config.field_hidden = height / 2;
            }
            ConfigMessage::QueueLen(queue_len) => self.config.queue_len = queue_len,

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
                    { Self::range_input("Field zoom", 0.25, 4.0, 0.01, config.field_zoom, field_zoom_callback) }

                    { Self::section_heading("Playfield") }
                    { Self::range_input("Field width", 4, 100, 1, config.field_width, field_width_callback) }
                    { Self::range_input("Field height", 6, 100, 1, config.field_height, field_height_callback) }
                    { Self::range_input("Queue length", 0, 7, 1, config.queue_len, queue_len_callback) }

                    { Self::section_heading("Handling") }
                    { Self::range_input("DAS", 0, 500, 1, config.delayed_auto_shift, das_callback) }
                    { Self::range_input("ARR", 0, 500, 1, config.auto_repeat_rate, arr_callback) }
                    { Self::range_input("SDR", 0, 500, 1, config.soft_drop_rate, sdr_callback) }
                </div>
            </div>
        }
    }
}
