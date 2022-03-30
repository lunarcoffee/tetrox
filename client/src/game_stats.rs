use tetrox::{field::LineClear, tetromino::SrsTetromino};
use yew::{html, Html};

use crate::animation::{Animation, AnimationData, AnimationState};

pub struct GameStatsDrawer {
    line_clear_text: String,
}

impl GameStatsDrawer {
    pub fn new() -> Self {
        GameStatsDrawer {
            line_clear_text: "".to_string(),
        }
    }

    pub fn get_html(&self, animation_state: &AnimationState) -> Html {
        let (line_clear_text_opacity, line_clear_letter_spacing) = animation_state
            .get_state(Animation::LineClearText)
            .and_then(|d| d.extract_float2())
            .unwrap_or((1.0, 0.1));

        let line_clear_text_style = format!(
            "opacity: {}; letter-spacing: {}rem;",
            line_clear_text_opacity, line_clear_letter_spacing
        );

        // TODO: make a helper that encapsulates this
        let (perfect_clear_text_opacity, perfect_clear_letter_spacing) = animation_state
            .get_state(Animation::PerfectClearText)
            .and_then(|d| d.extract_float2())
            .unwrap_or((0.0, 0.1));

        let perfect_clear_text_style = format!(
            "opacity: {}; letter-spacing: {}rem;",
            perfect_clear_text_opacity, perfect_clear_letter_spacing
        );

        html! {
            <div class="game-stats">
                <p class="game-stats-clear-text" style={ line_clear_text_style }>
                    { &self.line_clear_text }
                </p>
                <p class="game-stats-clear-text bold" style={ perfect_clear_text_style }>
                    { "perfect clear" }
                </p>
            </div>
        }
    }

    pub fn set_clear_type(&mut self, animation_state: &mut AnimationState, clear_type: LineClear<SrsTetromino>) {
        let n_lines = clear_type.n_lines();

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
    }

    fn register_text_animation(animation_state: &mut AnimationState, animation: Animation) {
        animation_state.set_state(animation, AnimationData::Float2(1.0, 0.1));
        animation_state.set_updater(animation, Self::text_animation_updater);
        animation_state.start_animation(animation);
    }

    // accelerating fade animation and decelerating expand animation
    fn text_animation_updater(data: &AnimationData) -> Option<AnimationData> {
        match data {
            AnimationData::Float2(opacity, letter_spacing) => (*opacity > 0.0).then(|| {
                AnimationData::Float2(
                    opacity * (opacity * (1.0 - 1e-5)).powf(0.15),
                    letter_spacing + 5e-4 * (1.0 / letter_spacing),
                )
            }),
            _ => None,
        }
    }
}
