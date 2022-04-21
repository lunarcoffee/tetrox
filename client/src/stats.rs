use std::time::Duration;

use sycamore::{
    component, easing,
    generic_node::Html,
    motion::create_tweened_signal,
    prelude::{create_effect, create_memo, create_signal, Scope, Signal},
    view,
    view::View,
    Prop,
};
use tetrox::{field::LineClear, PieceKind};

#[derive(Prop)]
pub struct StatsProps<'a, P: PieceKind> {
    last_line_clear: &'a Signal<Option<LineClear<P>>>,
}

#[component]
pub fn Stats<'a, P: PieceKind, G: Html>(cx: Scope<'a>, props: StatsProps<'a, P>) -> View<G> {
    let (lc_text, lc_view) = styled_text(cx, "clear-text", 0.0, 0.5);
    let (pc_text, pc_view) = styled_text(cx, "clear-text", 0.0, 0.5);
    let (combo_text, combo_view) = styled_text(cx, "combo-text", 0.5, 0.25);
    let (b2b_text, b2b_view) = styled_text(cx, "b2b-text", 0.5, 0.25);

    create_effect(cx, || {
        let line_clear = props.last_line_clear.get();
        line_clear
            .as_ref()
            .as_ref()
            .and_then(|l| {
                // update the line clear text if lines were cleared or the last locked piece was a spin
                (l.n_lines() > 0 || l.spin().is_some()).then(|| {
                    let mini = if l.is_mini() { "mini" } else { "" };
                    let spin = l.spin().map(|_| "t-spin ").unwrap_or("");
                    let n_text = ["", "single", "double", "triple", "quad"][l.n_lines()];
                    format!("{}{}{}", mini, spin, n_text).trim().to_string()
                })
            })
            .map(|t| lc_text.set(t));
    });

    create_effect(cx, || {
        let line_clear = props.last_line_clear.get();
        line_clear
            .as_ref()
            .as_ref()
            .and_then(|l| l.is_perfect_clear().then(|| "perfect clear"))
            .map(|t| pc_text.set(t.to_string()));
    });

    let combo = create_signal(cx, 0);
    let b2b = create_signal(cx, 0);

    // update combo and b2b
    create_effect(cx, || {
        props.last_line_clear.get().as_ref().as_ref().map(|l| {
            let old_combo = *combo.get();
            let old_b2b = *b2b.get();

            if l.n_lines() > 0 {
                combo.set(*combo.get() + 1);

                // quad or higher or spin keeps b2b
                if l.n_lines() >= 4 || l.spin().is_some() {
                    b2b.set(*b2b.get() + 1);
                } else {
                    b2b.take();
                }
            } else {
                combo.take();
            }

            // update combo and b2b text if the values changed
            if old_combo != *combo.get() {
                combo_text.set(format!("{}x combo", combo.get()));
            }
            if old_b2b != *b2b.get() {
                b2b_text.set(format!("{}x b2b", b2b.get()));
            }
        });
    });

    view! { cx, (lc_view) (pc_view) (combo_view) (b2b_view) }
}

// returns the signal for accessing the text, the corresponding view with the dynamic styles applied, a signal for
// whether the text animation should be reset, and the callback to reset the animation
fn styled_text<'a, G: Html>(cx: Scope<'a>, class: &'a str, ls_add: f64, ls_mul: f64) -> (&'a Signal<String>, View<G>) {
    let text = create_signal(cx, "".to_string());

    // updating the text causes `show_text` to become true, which will be checked by the animation reset effect
    let show_text = create_signal(cx, false);
    create_effect(cx, || {
        text.track();
        show_text.set(true);
    });

    // dynamic style values for animation
    let opacity = create_tweened_signal(cx, 1.0f64, Duration::from_millis(1_500), easing::quart_in);
    let spacing = create_tweened_signal(cx, 0.0f64, Duration::from_millis(1_500), easing::cubic_out);

    let opacity_style = create_memo(cx, || format!("opacity: {}%;", *opacity.get() * 100.0));
    let ls_style = create_memo(cx, move || format!("letter-spacing: {}rem;", *spacing.get() * ls_mul));
    let style = create_memo(cx, || format!("{}{}", opacity_style.get(), ls_style.get()));

    // resetting (running) the animation turns `show_text` false, preventing the weird loop thing
    let reset_style_animation = move || {
        opacity.signal().set(1.0);
        opacity.set(0.0);
        spacing.signal().set(ls_add);
        spacing.set(1.0);
        show_text.set(false);
    };

    // reset the animation if the text is updated
    // using `text` directly here causes the animation to loop? not sure why
    create_effect(cx, move || {
        if *show_text.get() {
            reset_style_animation();
        }
    });

    let view = view! { cx, p(class=class, style=style.get()) { (text.get()) } };
    (text, view)
}