use std::ops::Deref;

use crate::board::Board;
use yew::{html, Component, Context, Html};

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
    DelayedAutoShift(u32),
    AutoRepeatRate(u32),
    SoftDropRate(u32),
}

pub struct ConfigPanelWrapper {
    config: Config,
}

impl Component for ConfigPanelWrapper {
    type Message = ConfigMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        ConfigPanelWrapper {
            config: Config::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ConfigMessage::DelayedAutoShift(das) => self.config.delayed_auto_shift = das,
            ConfigMessage::AutoRepeatRate(arr) => self.config.auto_repeat_rate = arr,
            ConfigMessage::SoftDropRate(sdr) => self.config.soft_drop_rate = sdr,
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="content">
                <Board config={ ReadOnlyConfig(self.config.clone()) }/>
                <div class="config-panel">
                    {"mod spods podasp odiapsodi paoid poas idpoasipodiaspodi aspodi aspodiaspodiapodi"}
                    // <input>
                </div>
            </div>
        }
    }
}
