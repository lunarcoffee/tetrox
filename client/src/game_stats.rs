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

    pub fn game_stats_html(&self, animation_state: &AnimationState) -> Html {
        let line_clear_text_opacity = animation_state
            .get_state(Animation::LineClearTextFade)
            .map(|AnimationData::Float(o)| *o)
            .unwrap_or(1.0);

        let perfect_clear_text_opacity = animation_state
            .get_state(Animation::PerfectClearTextFade)
            .map(|AnimationData::Float(o)| *o)
            .unwrap_or(0.0);

        html! {
            <div class="game-stats">
                <p class="game-stats-clear-text"
                    style={ format!("opacity: {};", line_clear_text_opacity) }>
                    { &self.line_clear_text }
                </p>
                <p class="game-stats-clear-text"
                    style={ format!("opacity: {};", perfect_clear_text_opacity) }>
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
            Self::register_fade_text_animation(animation_state, Animation::LineClearTextFade);

            if clear_type.is_perfect_clear() {
                Self::register_fade_text_animation(animation_state, Animation::PerfectClearTextFade);
            }
        }
    }

    fn register_fade_text_animation(animation_state: &mut AnimationState, animation: Animation) {
        animation_state.set_state(animation, AnimationData::Float(1.0));
        animation_state.set_updater(animation, Self::fade_animation_updater);
        animation_state.start_animation(animation);
    }

    // accelerating fade animation
    fn fade_animation_updater(AnimationData::Float(opacity): &AnimationData) -> Option<AnimationData> {
        (*opacity > 0.0).then(|| AnimationData::Float(opacity * (opacity * (1.0 - 1e-5)).powf(0.15)))
    }
}
