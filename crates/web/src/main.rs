mod stepper;

use std::sync::Arc;

use leptos::ev;
use leptos::prelude::*;
use rankfast::estimate_turns;
use stepper::{Step, Stepper};

/// Reads the URL hash fragment and parses it into an answer history.
///
/// Each `a` character maps to `true` (left is better), each `b` maps
/// to `false`. All other characters are silently ignored.
fn read_hash_answers() -> Vec<bool> {
    let hash = window().location().hash().unwrap_or_default();

    hash.chars()
        .filter_map(|c| match c {
            'a' => Some(true),
            'b' => Some(false),
            _ => None,
        })
        .collect()
}

/// Pushes the answer history to the URL hash as a new history entry.
///
/// Each `true` becomes `a`, each `false` becomes `b`.
fn push_hash(answers: &[bool]) {
    let hash: String = answers.iter().map(|&b| if b { 'a' } else { 'b' }).collect();

    let win = window();
    if let Ok(h) = win.history() {
        let url = format!("#{hash}");
        let _ = h.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
    }
}

/// Replays the answer sequence through a fresh stepper and returns
/// the resulting UI state.
fn derive_state(n: usize, answers: &[bool]) -> RankState {
    let mut stepper = Stepper::new(n);
    let mut last_step = stepper.step();

    for &answer in answers {
        if last_step == Step::Done {
            break;
        }
        last_step = stepper.answer(answer);
    }

    match last_step {
        Step::Compare { a, b } => RankState {
            current: Some((a, b)),
            ranking: None,
            comparisons: stepper.comparisons_made(),
        },
        Step::Done => RankState {
            current: None,
            ranking: stepper.take_order(),
            comparisons: stepper.comparisons_made(),
        },
    }
}

#[derive(Clone, PartialEq)]
struct RankState {
    current: Option<(usize, usize)>,
    ranking: Option<Vec<usize>>,
    comparisons: usize,
}

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

    let n = items.len();
    let estimate = estimate_turns(n);

    // The URL hash is the source of truth. This signal mirrors it.
    let (answers, set_answers) = signal(read_hash_answers());

    // All UI state is derived from the answer history.
    let state = Memo::new(move |_| derive_state(n, &answers.get()));

    // Sync URL → signal on back/forward and manual hash edits.
    let _popstate = window_event_listener(ev::popstate, move |_| {
        set_answers.set(read_hash_answers());
    });
    let _hashchange = window_event_listener(ev::hashchange, move |_| {
        set_answers.set(read_hash_answers());
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
                        {move || state.get().comparisons} " / " {estimate}
                    </span>
                </div>
                <div class="progress-bar">
                    <div
                        class="progress-fill"
                        style:width=move || {
                            let pct = if estimate > 0 {
                                100 * state.get().comparisons / estimate
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
                    let s = state.get();
                    match (s.ranking, s.current) {
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
                            let on_a = move |_| {
                                set_answers.update(|ans| {
                                    ans.push(true);
                                    push_hash(ans);
                                });
                            };
                            let on_b = move |_| {
                                set_answers.update(|ans| {
                                    ans.push(false);
                                    push_hash(ans);
                                });
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
