use tetrox::{field::LineClear, tetromino::SrsTetromino};
use yew::{html, Context, Html};

use crate::board::{Board, BoardTimers};

pub struct GameStatsDrawer {
    line_clear_text: String,

    line_clear_text_opacity: f64,
    perfect_clear_text_opacity: f64,
}

impl GameStatsDrawer {
    pub fn new() -> Self {
        GameStatsDrawer {
            line_clear_text: "".to_string(),

            line_clear_text_opacity: 1.0,
            perfect_clear_text_opacity: 0.0,
        }
    }

    pub fn game_stats_html(&self) -> Html {
        html! {
            <div class="game-stats">
                <p class="game-stats-clear-text"
                    style={ format!("opacity: {};", self.line_clear_text_opacity) }>
                    { &self.line_clear_text }
                </p>
                <p class="game-stats-clear-text"
                    style={ format!("opacity: {};", self.perfect_clear_text_opacity) }>
                    { "perfect clear" }
                </p>
            </div>
        }
    }

    pub fn set_clear_type(
        &mut self,
        ctx: &Context<Board>,
        timers: &mut BoardTimers,
        clear_type: LineClear<SrsTetromino>,
    ) {
        let n_lines = clear_type.n_lines();

        if n_lines > 0 || clear_type.spin().is_some() {
            let mini = clear_type.is_mini().then(|| "mini ").unwrap_or("");
            let spin = clear_type.spin().map(|_| "t-spin ").unwrap_or("");
            let n_text = ["", "single ", "double ", "triple ", "quad "][n_lines];
            self.line_clear_text = format!("{}{}{}", mini, spin, n_text).trim().to_string();

            self.line_clear_text_opacity = 1.0;
            timers.fade_clear_text(ctx);

            if clear_type.is_perfect_clear() {
                self.perfect_clear_text_opacity = 1.0;
                timers.fade_perfect_clear_text(ctx);
            }
        }
    }

    // TODO: maybe make this fade all text
    pub fn fade_clear_type(&mut self, timers: &mut BoardTimers) {
        if self.line_clear_text_opacity > 0.0 {
            self.line_clear_text_opacity *= (self.line_clear_text_opacity * (1.0 - 1e-5)).powf(0.15);
        } else {
            timers.cancel_clear_text();
        }
    }

    pub fn fade_perfect_clear(&mut self, timers: &mut BoardTimers) {
        if self.perfect_clear_text_opacity > 0.0 {
            self.perfect_clear_text_opacity *= (self.perfect_clear_text_opacity * (1.0 - 1e-5)).powf(0.15);
        } else {
            timers.cancel_perfect_clear_text();
        }
    }
}
