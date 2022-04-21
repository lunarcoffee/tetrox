use tetrox::{field::LineClear, tetromino::SrsTetromino};
use yew::{html, Html};

use crate::animation::{Animation, AnimationData, AnimationState};

pub struct GameStatsDrawer {
    line_clear_text: String,
    back_to_back_text: String,
    combo_text: String,

    back_to_back: usize,
    combo: usize,
}

impl GameStatsDrawer {
    pub fn new() -> Self {
        GameStatsDrawer {
            line_clear_text: "".to_string(),
            back_to_back_text: "".to_string(),
            combo_text: "".to_string(),

            back_to_back: 0,
            combo: 0,
        }
    }

    pub fn get_html(&self, state: &AnimationState) -> Html {
        let line_clear_style = Self::animated_text_style(state, Animation::LineClearText, (1.0, 0.1));
        let perfect_clear_style = Self::animated_text_style(state, Animation::PerfectClearText, (0.0, 0.1));
        let back_to_back_style = Self::animated_text_style(state, Animation::BackToBackText, (1.0, 0.01));
        let combo_style = Self::animated_text_style(state, Animation::ComboText, (1.0, 0.01));

        html! {
            <div class="game-stats">
                <p class="game-stats-clear-text" style={ line_clear_style }>{ &self.line_clear_text }</p>
                <p class="game-stats-clear-text bold" style={ perfect_clear_style }>{ "perfect clear" }</p>
                <p class="game-stats-combo-text" style={ combo_style }>{ &self.combo_text }</p>
                <p class="game-stats-b2b-text bold" style={ back_to_back_style }>{ &self.back_to_back_text }</p>
            </div>
        }
    }

    fn animated_text_style(animation_state: &AnimationState, animation: Animation, default: (f64, f64)) -> String {
        let (opacity, letter_spacing) =
            animation_state.extract_state(animation, AnimationData::extract_float2, default);
        format!("opacity: {}; letter-spacing: {}rem;", opacity, letter_spacing)
    }

    pub fn set_clear_type(&mut self, animation_state: &mut AnimationState, clear_type: LineClear<SrsTetromino>) {
        let n_lines = clear_type.n_lines();

        // update clear type and pc text
        if n_lines > 0 || clear_type.spin().is_some() {
            let mini = clear_type.is_mini().then(|| "mini ").unwrap_or("");
            let spin = clear_type.spin().map(|_| "t-spin ").unwrap_or("");
            let n_text = ["", "single ", "double ", "triple ", "quad "][n_lines];

            self.line_clear_text = format!("{}{}{}", mini, spin, n_text).trim().to_string();
            Self::register_text_animation(animation_state, Animation::LineClearText);

            if clear_type.is_perfect_clear() {
                Self::register_text_animation(animation_state, Animation::PerfectClearText);
            }
        }

        let old_back_to_back = self.back_to_back;
        let old_combo = self.combo;

        // update back to back, combo, and corresponding text
        if n_lines > 0 {
            // quads and higher and line-clearing spins keep back to back
            if n_lines >= 4 || clear_type.spin().is_some() {
                self.back_to_back += 1;
            } else {
                self.back_to_back = 0;
            }
            self.combo += 1;
        } else {
            self.combo = 0;
        }

        // only show back to back and combo text if it is not zero or if it was just reset
        if old_back_to_back != self.back_to_back {
            self.back_to_back_text = format!("back to back: {}", self.back_to_back);
            Self::register_text_animation(animation_state, Animation::BackToBackText);
        }
        if old_combo != self.combo {
            self.combo_text = format!("combo: {}", self.combo);
            Self::register_text_animation(animation_state, Animation::ComboText);
        }
    }

    fn register_text_animation(animation_state: &mut AnimationState, animation: Animation) {
        animation_state.set_state(animation, AnimationData::Float2(1.0, 0.1));
        animation_state.set_updater(animation, Self::text_animation_updater);
        animation_state.start_animation(animation);
    }

    // accelerating fade animation and decelerating expand animation
    fn text_animation_updater(data: &AnimationData) -> Option<AnimationData> {
        match data {
            AnimationData::Float2(opacity, letter_spacing) if *opacity > 0.0 => Some(AnimationData::Float2(
                opacity * (opacity * (1.0 - 1e-5)).powf(0.15),
                letter_spacing + 5e-4 * (1.0 / letter_spacing),
            )),
            _ => None,
        }
    }
}
