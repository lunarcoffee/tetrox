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
        let field_width_callback = make_update_callback!(HtmlInputElement, ConfigMessage::FieldWidth);
        let field_height_callback = make_update_callback!(HtmlInputElement, ConfigMessage::FieldHeight);
        let queue_len_callback = make_update_callback!(HtmlInputElement, ConfigMessage::QueueLen);
        let das_callback = make_update_callback!(HtmlInputElement, ConfigMessage::DelayedAutoShift);
        let arr_callback = make_update_callback!(HtmlInputElement, ConfigMessage::AutoRepeatRate);
        let sdr_callback = make_update_callback!(HtmlInputElement, ConfigMessage::SoftDropRate);

        let skin_name = &self.config.skin_name;
        let field_width = self.config.field_width;
        let field_height = self.config.field_height;
        let queue_len = self.config.queue_len;
        let das = self.config.delayed_auto_shift.to_string();
        let arr = self.config.auto_repeat_rate.to_string();
        let sdr = self.config.soft_drop_rate.to_string();

        html! {
            <div class="content">
                <Board config={ ReadOnlyConfig(self.config.clone()) }/>
                <div class="config-panel">
                    <p class="config-heading">{ "Options" }</p>
                    <p class="config-option-label">{ format!("Block skin ({}):", skin_name) }</p>
                    <select oninput={ skin_name_callback }>{
                        for crate::SKIN_NAMES.iter().map(|skin| {
                            html! { <option value={ *skin } selected={ skin_name == *skin }>{ skin }</option> }
                        })
                    }</select>

                    <p class="config-option-label">{ format!("Field width ({}):", field_width) }</p>
                    <input type="range" min="4" max="100" value={ field_width.to_string() }
                           oninput={ field_width_callback }/>

                    <p class="config-option-label">{ format!("Field height ({}):", field_height) }</p>
                    <input type="range" min="6" max="100" value={ field_height.to_string() }
                           oninput={ field_height_callback }/>

                    <p class="config-option-label">{ format!("Queue length ({}):", queue_len) }</p>
                    <input type="range" min="0" max="7" value={ queue_len.to_string() } oninput={ queue_len_callback }/>

                    <p class="config-option-label">{ format!("DAS ({} ms):", das) }</p>
                    <input type="range" min="0" max="500" value={ das } oninput={ das_callback }/>

                    <p class="config-option-label">{ format!("ARR ({} ms):", arr) }</p>
                    <input type="range" min="0" max="500" value={ arr } oninput={ arr_callback }/>

                    <p class="config-option-label">{ format!("SDR ({} ms):", sdr) }</p>
                    <input type="range" min="0" max="500" value={ sdr } oninput={ sdr_callback }/>
                </div>
            </div>
        }
    }
}
