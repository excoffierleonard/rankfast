mod stepper;

use std::sync::Arc;

use leptos::mount::mount_to_body;
use leptos::prelude::*;
use rankfast::estimate_turns;
use stepper::{Step, Stepper};

fn main() {
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let items = Arc::new(vec![
        "Blue".to_string(),
        "Orange".to_string(),
        "Red".to_string(),
        "Black".to_string(),
        "Green".to_string(),
        "Yellow".to_string(),
        "Purple".to_string(),
        "White".to_string(),
    ]);

    let estimate = estimate_turns(items.len());

    let mut stepper = Stepper::new(items.len());
    let initial = stepper.step();
    let (current_init, ranking_init) = match initial {
        Step::Compare { a, b } => (Some((a, b)), None),
        Step::Done => (None, stepper.take_order()),
    };

    let (current, set_current) = signal(current_init);
    let (ranking, set_ranking) = signal::<Option<Vec<usize>>>(ranking_init);
    let (comparisons, set_comparisons) = signal(stepper.comparisons_made());
    let (_, set_stepper) = signal(stepper);

    let apply_answer = Arc::new(move |better_is_a: bool| {
        set_stepper.update(|stepper| {
            let step = stepper.answer(better_is_a);
            set_comparisons.set(stepper.comparisons_made());
            match step {
                Step::Compare { a, b } => {
                    set_current.set(Some((a, b)));
                    set_ranking.set(None);
                }
                Step::Done => {
                    set_current.set(None);
                    let order = stepper.take_order().unwrap_or_default();
                    set_ranking.set(Some(order));
                }
            }
        });
    });

    view! {
        <main class="app">
            <header class="header">
                <h1>"Rankfast"</h1>
                <p class="subtitle">"Pairwise ranking tool"</p>
            </header>

            <div class="progress-area">
                <div class="progress-text">
                    <span>"Comparison"</span>
                    <span class="progress-numbers">
                        {move || comparisons.get()} " / " {estimate}
                    </span>
                </div>
                <div class="progress-bar">
                    <div
                        class="progress-fill"
                        style:width=move || {
                            let pct = if estimate > 0 {
                                100 * comparisons.get() / estimate
                            } else {
                                100
                            };
                            format!("{pct}%")
                        }
                    />
                </div>
            </div>

            {
                let items = items.clone();
                move || {
                    let items = items.clone();
                    match (ranking.get(), current.get()) {
                        (Some(order), _) => view! {
                            <section class="results">
                                <h2 class="results-title">"Your Ranking"</h2>
                                <ol class="ranking-list">
                                    {order
                                        .iter()
                                        .enumerate()
                                        .map(|(rank, &idx)| {
                                            view! {
                                                <li
                                                    class="ranking-item"
                                                    class:gold={rank == 0}
                                                    class:silver={rank == 1}
                                                    class:bronze={rank == 2}
                                                >
                                                    <span class="rank-number">{rank + 1}</span>
                                                    <span class="rank-name">
                                                        {items[idx].clone()}
                                                    </span>
                                                </li>
                                            }
                                        })
                                        .collect_view()}
                                </ol>
                            </section>
                        }
                        .into_any(),
                        (None, Some((a, b))) => {
                            let on_a = {
                                let apply_answer = apply_answer.clone();
                                move |_| apply_answer(true)
                            };
                            let on_b = {
                                let apply_answer = apply_answer.clone();
                                move |_| apply_answer(false)
                            };

                            view! {
                                <section class="compare">
                                    <h2 class="compare-prompt">"Which do you prefer?"</h2>
                                    <div class="compare-buttons">
                                        <button class="choice-btn" on:click=on_a>
                                            {items[a].clone()}
                                        </button>
                                        <span class="vs">"vs"</span>
                                        <button class="choice-btn" on:click=on_b>
                                            {items[b].clone()}
                                        </button>
                                    </div>
                                </section>
                            }
                            .into_any()
                        }
                        _ => view! {
                            <section class="results">
                                <p class="no-compare">
                                    "Only one item \u{2014} no comparisons needed!"
                                </p>
                            </section>
                        }
                        .into_any(),
                    }
                }
            }

            <section class="items">
                <h3 class="items-heading">"Items being ranked"</h3>
                <div class="items-tags">
                    {items
                        .iter()
                        .map(|name| {
                            view! { <span class="item-tag">{name.clone()}</span> }
                        })
                        .collect_view()}
                </div>
            </section>
        </main>
    }
}
