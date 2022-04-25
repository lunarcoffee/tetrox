use std::{
    cell::RefCell,
    fmt::{self, Display},
    ops::Deref,
    str::FromStr,
    time::Duration,
};

use crate::{
    menu::Menu,
    util::{self, Padding, SectionHeading},
};

use bimap::BiMap;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use sycamore::{
    component, easing,
    generic_node::Html,
    motion::create_tweened_signal,
    prelude::{create_effect, create_memo, create_signal, provide_context_ref, Keyed, ReadSignal, Scope, Signal},
    view,
    view::View,
    Prop,
};

use tetrox::{
    kicks::{AscKickTable, BasicKickTable, KickTable, KickTable180, SrsKickTable, TetrIo180KickTable},
    pieces::{
        mino123::Mino123,
        mino1234::Mino1234,
        tetromino::{TetrominoAsc, TetrominoSrs},
        PieceKind, PieceKindTrait,
    },
    spins::{ImmobileSpinDetector, NoSpinDetector, SpinDetector, TSpinDetector},
};
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, HtmlSelectElement, KeyboardEvent, Storage};

const CONFIG_LOCAL_STORAGE_KEY: &str = "config";

#[component]
pub fn ConfigPanel<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let c = Config::from_local_storage(get_local_storage()).unwrap_or_else(|| Config::default());

    // separate signal for values required by canvas drawing because if both the canvas and the drawer directly used
    // the config, sometimes the view's tracked signals would update after the canvas drawer effect's, leading the
    // drawer to use an invalid `NodeRef`
    let field_values = FieldValues::new(c.field_width, c.field_height, c.field_hidden, c.queue_len);
    let field_values = create_signal(cx, field_values);
    provide_context_ref(cx, field_values.map(cx, |d| d.clone()));

    let config = create_signal(cx, RefCell::new(c));
    provide_context_ref(cx, config);

    // store the config on changes
    create_effect(cx, move || {
        let json = serde_json::to_string(&*config.get()).unwrap();
        get_local_storage().set_item(CONFIG_LOCAL_STORAGE_KEY, &json).unwrap();
    });

    let updater = move |msg| {
        // see comment on `field_values` above
        match msg {
            // these are the only messages that would require a canvas update
            ConfigMsg::FieldWidth(width) => field_values.modify().width = width,
            ConfigMsg::FieldHidden(hidden) => {
                field_values.modify().height = hidden * 2;
                field_values.modify().hidden = hidden;
            }
            ConfigMsg::QueueLen(queue_len) => field_values.modify().queue_len = queue_len,
            _ => {}
        }

        // untracked so this isn't called on every config update
        util::with_signal_mut_untracked(config, |config| {
            if let ConfigMsg::FieldHidden(hidden) = msg {
                config.field_height = hidden * 2;
                config.field_hidden = hidden;
            }

            // match statement for updating each config value given its message
            macro_rules! gen_config_setter_match {
                ($($fields:ident; $msgs:ident),+) => { match msg {
                    $(ConfigMsg::$msgs(ref new_value) => config.$fields = new_value.clone(),)*
                    _ => {}
                } }
            }
            gen_config_setter_match! {
                gravity_delay; GravityDelay, lock_delay; LockDelay, move_limit; MoveLimit,
                topping_out_enabled; ToppingOutEnabled, auto_lock_enabled; AutoLockEnabled,
                gravity_enabled; GravityEnabled, move_limit_enabled; MoveLimitEnabled, field_width; FieldWidth,
                queue_len; QueueLen, piece_type; PieceType, spin_types; SpinType, kick_table; KickTable,
                kick_table_180; KickTable180, goal_type; GoalType, goal_n_lines; GoalNLines,
                goal_time_limit_secs; GoalTimeLimitSecs, skin_name; SkinName, field_zoom; FieldZoom,
                vertical_offset; VerticalOffset, shadow_opacity; ShadowOpacity, keybinds; Keybinds,
                delayed_auto_shift; DelayedAutoShift, auto_repeat_rate; AutoRepeatRate, soft_drop_rate; SoftDropRate
            }
        });
    };

    // make config value signals and effects which update the config when the value signal is changed
    macro_rules! gen_config_signals {
        ($($field:ident; $msg:ident),+) => { $(
            let $field = create_signal(cx, config.get().borrow().$field.clone());
            create_effect(cx, move || updater(ConfigMsg::$msg((*$field.get()).clone())));

            // react to external config updates
            let selector = util::create_config_selector(cx, config, |c| c.$field.clone());
            create_effect(cx, || $field.set((*selector.get()).clone()));
        )* }
    }
    gen_config_signals! {
        gravity_delay; GravityDelay, lock_delay; LockDelay, move_limit; MoveLimit,
        topping_out_enabled; ToppingOutEnabled, auto_lock_enabled; AutoLockEnabled, gravity_enabled; GravityEnabled,
        move_limit_enabled; MoveLimitEnabled, field_width; FieldWidth, field_hidden; FieldHidden, queue_len; QueueLen,
        piece_type; PieceType, spin_types; SpinType, kick_table; KickTable, kick_table_180; KickTable180,
        goal_type; GoalType, goal_n_lines; GoalNLines, goal_time_limit_secs; GoalTimeLimitSecs, skin_name; SkinName,
        field_zoom; FieldZoom, vertical_offset; VerticalOffset, shadow_opacity; ShadowOpacity, keybinds; Keybinds,
        delayed_auto_shift; DelayedAutoShift, auto_repeat_rate; AutoRepeatRate, soft_drop_rate; SoftDropRate
    };

    // make label and item pair list for the select inputs
    macro_rules! gen_selector_items {
        ($enum_name:ident, $($item_label:expr),*) => {
            [$($item_label,)*].into_iter().zip($enum_name::iter()).collect()
        }
    }
    let piece_kind_items = gen_selector_items!(PieceTypes, "Tetromino SRS", "Tetromino ASC", "123Mino", "1234Mino");
    let kick_table_items = gen_selector_items!(KickTables, "SRS", "ASC", "Basic");
    let kick_table_180_items = gen_selector_items!(KickTable180s, "TETR.IO", "Basic");
    let spin_type_items = gen_selector_items!(SpinTypes, "T-Spins", "Immobile", "None");
    let goal_type_items = gen_selector_items!(GoalTypes, "None", "Lines cleared", "Time limit");
    let skin_name_items = ["Tetrox", "Gradient", "Inset", "Cirxel", "TETR.IO", "Solid"]
        .into_iter()
        .zip(crate::SKIN_NAMES.iter().map(|s| s.to_string()))
        .collect();

    macro_rules! keybind_capture_buttons {
        ($($label:expr; $input:ident),*) => { view! { cx,
            div(class="menu-button-box") {
                $(InputCaptureButton { label: $label, input: Input::$input, keybinds })*
            }
        } }
    }

    let ui_offset = create_tweened_signal(cx, 0.0, Duration::from_millis(200), easing::quart_inout);
    let config_style = create_memo(cx, || format!("margin-right: -{}rem;", ui_offset.get()));

    let ui_enabled = create_signal(cx, UiEnabled(true));
    provide_context_ref(cx, ui_enabled);
    create_effect(cx, || ui_offset.set(if **ui_enabled.get() { 0.0 } else { 20.0 }));

    view! { cx,
        div(class="content") {
            Menu { ui_offset }

            div(class="config-panel", style=config_style.get()) {
                SectionHeading("Gameplay")
                RangeInput { label: "Gravity delay", min: 0, max: 5_000, step: 5, value: gravity_delay }
                RangeInput { label: "Lock delay", min: 10, max: 3_000, step: 5, value: lock_delay }
                RangeInput { label: "Move limit", min: 1, max: 100, step: 1, value: move_limit }
                div(class="menu-button-box") {
                    ToggleButton { label: "Topping out", value: topping_out_enabled }
                    ToggleButton { label: "Lock delay", value: auto_lock_enabled }
                    ToggleButton { label: "Gravity", value: gravity_enabled }
                    ToggleButton { label: "Move limit", value: move_limit_enabled }
                }
                Padding(2)

                SectionHeading("Playfield")
                RangeInput { label: "Field width", min: 4, max: 100, step: 1, value: field_width }
                RangeInput { label: "Field height", min: 3, max: 100, step: 1, value: field_hidden }
                RangeInput { label: "Queue length", min: 0, max: 7, step: 1, value: queue_len }
                SelectInput { label: "Piece kind", items: piece_kind_items, value: piece_type }
                SelectInput { label: "Spin detection", items: spin_type_items, value: spin_types }
                SelectInput { label: "Kick table", items: kick_table_items, value: kick_table }
                SelectInput { label: "180 kick table", items: kick_table_180_items, value: kick_table_180 }
                Padding(4)

                SectionHeading("Goal")
                SelectInput { label: "Goal type", items: goal_type_items, value: goal_type }
                Padding(4)
                (match *goal_type.get() {
                    GoalTypes::LinesCleared => view! { cx,
                        Padding(2)
                        RangeInput { label: "Lines cleared", min: 1, max: 1_000, step: 1, value: goal_n_lines }
                    },
                    GoalTypes::TimeLimit => view! { cx,
                        Padding(2)
                        RangeInput { label: "Time limit", min: 5, max: 3_600, step: 1, value: goal_time_limit_secs }
                    },
                    _ => view! { cx, }
                })

                SectionHeading("Visual")
                RangeInput { label: "Field zoom", min: 0.1, max: 4.0, step: 0.05, value: field_zoom }
                RangeInput { label: "Vertical offset", min: -2_000, max: 2_000, step: 10, value: vertical_offset }
                RangeInput { label: "Shadow opacity", min: 0.0, max: 1.0, step: 0.05, value: shadow_opacity }
                SelectInput { label: "Block skin", items: skin_name_items, value: skin_name }
                Padding(4)

                SectionHeading("Keybinds")
                (keybind_capture_buttons! {
                    "Left"; Left, "Right"; Right, "Soft drop"; SoftDrop, "Hard drop"; HardDrop,
                    "Rotate CW"; RotateCw, "Rotate CCW"; RotateCcw, "Rotate 180"; Rotate180, "Swap hold"; SwapHold,
                    "Reset"; Reset, "Show/hide UI"; ShowHideUi
                })
                Padding(2)

                SectionHeading("Handling")
                RangeInput { label: "DAS", min: 0, max: 500, step: 1, value: delayed_auto_shift }
                RangeInput { label: "ARR", min: 0, max: 500, step: 1, value: auto_repeat_rate }
                RangeInput { label: "SDR", min: 0, max: 500, step: 1, value: soft_drop_rate }
            }
        }
    }
}

fn get_local_storage() -> Storage { web_sys::window().unwrap().local_storage().unwrap().unwrap() }

#[derive(Prop)]
struct RangeInputProps<'a, T: Copy + Display + FromStr + 'static> {
    label: &'static str,
    min: T,
    max: T,
    step: T,
    value: &'a Signal<T>,
}

#[component]
fn RangeInput<'a, E, T, G>(cx: Scope<'a>, props: RangeInputProps<'a, T>) -> View<G>
where
    E: fmt::Debug,
    T: Copy + Display + FromStr<Err = E> + 'static,
    G: Html,
{
    let RangeInputProps {
        label,
        min,
        max,
        step,
        value,
    } = props;

    view! { cx,
        div(class="menu-option") {
            InputLabel { label, value }
            input(
                type="range",
                min=min, max=max, step=step, value=value.to_string(),
                on:input=|e: Event| {
                    let elem = e.target().unwrap().dyn_into::<HtmlInputElement>();
                    value.set(elem.unwrap().value().parse().unwrap());
                },
            )
        }
    }
}

#[derive(Prop)]
struct SelectInputProps<'a, T: Clone + PartialEq + Eq + 'static> {
    label: &'static str,
    items: Vec<(&'static str, T)>,
    value: &'a Signal<T>,
}

#[component]
fn SelectInput<'a, T, G>(cx: Scope<'a>, props: SelectInputProps<'a, T>) -> View<G>
where
    T: Clone + PartialEq + Eq + 'static,
    G: Html,
{
    let SelectInputProps { label, items, value } = props;
    let items = create_signal(cx, items);

    view! { cx,
        div(class="menu-option") {
            label(class="menu-option-label") { (label) ":" }
            select(
                on:input=|e: Event| {
                    let new_label = e.target().unwrap().dyn_into::<HtmlSelectElement>().unwrap().value();
                    value.set(items.get().iter().find(|i| i.0 == &new_label).unwrap().1.clone());
                },
            ) {
                Keyed {
                    iterable: items,
                    view: move |cx, (label, item)| view! { cx,
                        option(value=label, selected=*value.get() == item) { (label.to_string()) }
                    },
                    key: |item| item.0,
                }
            }
        }
    }
}

#[derive(Prop)]
struct ToggleButtonProps<'a> {
    label: &'static str,
    value: &'a Signal<bool>,
}

// button that toggles a `bool`
#[component]
fn ToggleButton<'a, G: Html>(cx: Scope<'a>, props: ToggleButtonProps<'a>) -> View<G> {
    let ToggleButtonProps { label, value } = props;
    let label = value.map(cx, move |v| format!("{} ({})", label, if *v { "on" } else { "off" }));

    view! { cx,
        div(class="menu-option") {
            input(
                type="button",
                class=format!("menu-toggle-button-{}", label),
                value=label.get(),
                on:click=|_| value.set(!*value.get()),
            )
        }
    }
}

#[derive(Prop)]
struct InputCaptureButtonProps<'a> {
    label: &'static str,
    input: Input,
    keybinds: &'a Signal<Keybinds>,
}

// button that captures keyboard input when pressed (used for assigning keybinds)
#[component]
fn InputCaptureButton<'a, G: Html>(cx: Scope<'a>, props: InputCaptureButtonProps<'a>) -> View<G> {
    let InputCaptureButtonProps { label, input, keybinds } = props;

    let is_capturing_input = create_signal(cx, false); // currently capturing input?
    let label = is_capturing_input.map(cx, move |i| {
        let keybind = i.then(|| "<press a key>".to_string()).unwrap_or_else(|| {
            keybinds
                .get()
                .get_by_left(&input)
                .map(|keybind| match keybind.as_str() {
                    " " => "Space",
                    _ if keybind.starts_with("Arrow") => &keybind[5..],
                    _ => keybind.as_str(),
                })
                .unwrap_or("<unset>")
                .to_string()
        });
        format!("{} ({})", label, keybind)
    });

    view! { cx,
        div(class="menu-option") {
            input(
                type="button",
                value=label.get(),
                on:click=|_| is_capturing_input.set(!*is_capturing_input.get()),
                on:keydown=move |e: Event| {
                    e.prevent_default();
                    let e = e.dyn_into::<KeyboardEvent>().unwrap();

                    // only change binds if currently capturing and let escape cancel the action
                    if *is_capturing_input.get() && !e.key().starts_with("Esc") {
                        keybinds.modify().insert(input, e.key());
                    }
                    is_capturing_input.set(false);
                },
            )
        }
    }
}

#[derive(Prop)]
struct InputLabelProps<'a, T: Display + 'static> {
    label: &'static str,
    value: &'a ReadSignal<T>,
}

#[component] // TODO: enable editing text
fn InputLabel<'a, T: Display + 'static, G: Html>(cx: Scope<'a>, props: InputLabelProps<'a, T>) -> View<G> {
    view! { cx, p(class="menu-option-label") { (props.label) " (" (props.value.get()) "):" } }
}

#[derive(Clone)]
pub struct FieldValues {
    pub width: usize,
    pub height: usize,
    pub hidden: usize,
    pub queue_len: usize,
}

impl FieldValues {
    pub fn new(width: usize, height: usize, hidden: usize, queue_len: usize) -> Self {
        FieldValues {
            width,
            height,
            hidden,
            queue_len,
        }
    }
}

// all types of `PieceKind`s
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum PieceTypes {
    TetrominoSrs,
    TetrominoAsc,
    Mino123,
    Mino1234,
}

impl PieceTypes {
    // get all `PieceKinds` of the `PieceType`
    pub fn kinds(&self) -> Vec<PieceKind> {
        match self {
            PieceTypes::TetrominoSrs => <TetrominoSrs as PieceKindTrait>::iter(),
            PieceTypes::TetrominoAsc => <TetrominoAsc as PieceKindTrait>::iter(),
            PieceTypes::Mino123 => <Mino123 as PieceKindTrait>::iter(),
            PieceTypes::Mino1234 => <Mino1234 as PieceKindTrait>::iter(),
        }
        .collect()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum KickTables {
    Srs,
    Asc,
    Basic,
}

impl KickTables {
    pub fn table(&self) -> &dyn KickTable {
        match self {
            KickTables::Srs => &SrsKickTable,
            KickTables::Asc => &AscKickTable,
            KickTables::Basic => &BasicKickTable,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum KickTable180s {
    TetrIo,
    Lru,
}

impl KickTable180s {
    pub fn table(&self) -> &dyn KickTable180 {
        match self {
            KickTable180s::TetrIo => &TetrIo180KickTable,
            KickTable180s::Lru => &BasicKickTable,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum SpinTypes {
    TSpins,
    AllImmobile,
    None,
}

impl SpinTypes {
    pub fn detector(&self) -> &dyn SpinDetector {
        match self {
            SpinTypes::TSpins => &TSpinDetector,
            SpinTypes::AllImmobile => &ImmobileSpinDetector,
            SpinTypes::None => &NoSpinDetector,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum GoalTypes {
    None,
    LinesCleared,
    TimeLimit,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter)]
pub enum Input {
    Left,
    Right,
    SoftDrop,
    HardDrop,
    RotateCw,
    RotateCcw,
    Rotate180,
    SwapHold,
    Reset,
    ShowHideUi,
}

pub type Keybinds = BiMap<Input, String>;

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    // gameplay
    pub gravity_delay: u32,
    pub lock_delay: u32,
    pub move_limit: usize,
    pub topping_out_enabled: bool,
    pub auto_lock_enabled: bool,
    pub gravity_enabled: bool,
    pub move_limit_enabled: bool,

    // field property settings
    pub field_width: usize,
    pub field_height: usize,
    pub field_hidden: usize,
    pub queue_len: usize,
    pub piece_type: PieceTypes,
    pub spin_types: SpinTypes,
    pub kick_table: KickTables,
    pub kick_table_180: KickTable180s,

    // goal settings
    pub goal_type: GoalTypes,
    pub goal_n_lines: u32,
    pub goal_time_limit_secs: u64,

    // visual settings
    pub skin_name: String,
    pub field_zoom: f64,
    pub vertical_offset: i32,
    pub shadow_opacity: f64,

    // controls
    pub keybinds: Keybinds,

    // handling
    pub delayed_auto_shift: u32,
    pub auto_repeat_rate: u32,
    pub soft_drop_rate: u32,
}

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
            (Input::SwapHold, "c"),
            (Input::Reset, "`"),
            (Input::ShowHideUi, "F9"),
        ];

        Config {
            gravity_delay: 1_000,
            lock_delay: 500,
            move_limit: 30,
            topping_out_enabled: true,
            auto_lock_enabled: true,
            gravity_enabled: true,
            move_limit_enabled: true,

            field_width: 10,
            field_height: 40,
            field_hidden: 20,
            queue_len: 5,
            piece_type: PieceTypes::TetrominoSrs,
            spin_types: SpinTypes::TSpins,
            kick_table: KickTables::Srs,
            kick_table_180: KickTable180s::TetrIo,

            goal_type: GoalTypes::None,
            goal_n_lines: 40,
            goal_time_limit_secs: 120,

            skin_name: "tetrox".to_string(),
            field_zoom: 1.0,
            vertical_offset: 170,
            shadow_opacity: 0.3,

            keybinds: inputs.into_iter().map(|(i, k)| (i, k.to_string())).collect(),

            delayed_auto_shift: 280,
            auto_repeat_rate: 50,
            soft_drop_rate: 30,
        }
    }
}

enum ConfigMsg {
    GravityDelay(u32),
    LockDelay(u32),
    MoveLimit(usize),
    ToppingOutEnabled(bool),
    AutoLockEnabled(bool),
    GravityEnabled(bool),
    MoveLimitEnabled(bool),

    FieldWidth(usize),
    FieldHidden(usize),
    QueueLen(usize),
    PieceType(PieceTypes),
    SpinType(SpinTypes),
    KickTable(KickTables),
    KickTable180(KickTable180s),

    GoalType(GoalTypes),
    GoalNLines(u32),
    GoalTimeLimitSecs(u64),

    SkinName(String),
    FieldZoom(f64),
    VerticalOffset(i32),
    ShadowOpacity(f64),

    Keybinds(Keybinds),

    DelayedAutoShift(u32),
    AutoRepeatRate(u32),
    SoftDropRate(u32),

    _ToggleUi,
}

pub struct UiEnabled(bool);

impl Deref for UiEnabled {
    type Target = bool;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<bool> for UiEnabled {
    fn from(b: bool) -> Self { UiEnabled(b) }
}
