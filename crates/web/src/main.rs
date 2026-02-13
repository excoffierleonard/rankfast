mod stepper;

use std::sync::Arc;

use leptos::ev;
use leptos::prelude::*;
use rankfast::estimate_turns;
use stepper::{Step, Stepper};

/// Parses the URL hash into items and answers.
///
/// Format: `#item1,item2,item3!aabba`
/// - Items are comma-separated, each URI-component-encoded
/// - `!` separates items from answers
/// - Answers are `a` (true) / `b` (false) chars
fn parse_hash() -> (Vec<String>, Vec<bool>) {
    let hash = window().location().hash().unwrap_or_default();
    let hash = hash.strip_prefix('#').unwrap_or(&hash);

    if hash.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let (items_part, answers_part) = match hash.split_once('!') {
        Some((i, a)) => (i, a),
        None => (hash, ""),
    };

    let items: Vec<String> = items_part
        .split(',')
        .map(decode_uri_component)
        .filter(|s| !s.is_empty())
        .collect();

    let answers: Vec<bool> = answers_part
        .chars()
        .filter_map(|c| match c {
            'a' => Some(true),
            'b' => Some(false),
            _ => None,
        })
        .collect();

    (items, answers)
}

/// Builds a URL hash string from items and answers.
fn build_hash(items: &[String], answers: &[bool]) -> String {
    let items_part: String = items
        .iter()
        .map(|s| encode_uri_component(s))
        .collect::<Vec<_>>()
        .join(",");

    if answers.is_empty() {
        return items_part;
    }

    let answers_part: String = answers.iter().map(|&b| if b { 'a' } else { 'b' }).collect();
    format!("{items_part}!{answers_part}")
}

/// Pushes the full state (items + answers) to the URL hash as a new history entry.
fn push_hash_full(items: &[String], answers: &[bool]) {
    let hash = build_hash(items, answers);
    let win = window();
    if let Ok(h) = win.history() {
        let url = format!("#{hash}");
        let _ = h.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
    }
}

fn encode_uri_component(s: &str) -> String {
    js_sys::encode_uri_component(s).into()
}

fn decode_uri_component(s: &str) -> String {
    js_sys::decode_uri_component(s).map_or_else(|_| s.to_string(), String::from)
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
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (initial_items, initial_answers) = parse_hash();

    let (items, set_items) = signal(initial_items);
    let (answers, set_answers) = signal(initial_answers);

    // All UI state is derived from the items + answer history.
    let state = Memo::new(move |_| {
        let cur_items = items.get();
        derive_state(cur_items.len(), &answers.get())
    });

    let estimate = Memo::new(move |_| estimate_turns(items.get().len()));

    // Sync URL -> signals on back/forward and manual hash edits.
    let _popstate = window_event_listener(ev::popstate, move |_| {
        let (new_items, new_answers) = parse_hash();
        set_items.set(new_items);
        set_answers.set(new_answers);
    });
    let _hashchange = window_event_listener(ev::hashchange, move |_| {
        let (new_items, new_answers) = parse_hash();
        set_items.set(new_items);
        set_answers.set(new_answers);
    });

    view! {
        <main class="app">
            <header class="header">
                <h1>"Rankfast"</h1>
                <p class="subtitle">"Pairwise ranking tool"</p>
            </header>

            {move || {
                let cur_items = items.get();
                if cur_items.is_empty() {
                    view! { <InputForm set_items set_answers /> }.into_any()
                } else {
                    let items_arc = Arc::new(cur_items);
                    let items_for_ranking = items_arc.clone();
                    let items_for_tags = items_arc.clone();
                    view! {
                        <div class="progress-area">
                            <div class="progress-text">
                                <span>"Comparison"</span>
                                <span class="progress-numbers">
                                    {move || state.get().comparisons} " / " {move || estimate.get()}
                                </span>
                            </div>
                            <div class="progress-bar">
                                <div
                                    class="progress-fill"
                                    style:width=move || {
                                        let est = estimate.get();
                                        let pct = if est > 0 {
                                            100 * state.get().comparisons / est
                                        } else {
                                            100
                                        };
                                        format!("{pct}%")
                                    }
                                />
                            </div>
                        </div>

                        {
                            let items_inner = items_for_ranking.clone();
                            move || {
                                let items_inner = items_inner.clone();
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
                                                                    {items_inner[idx].clone()}
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
                                                push_hash_full(&items.get(), ans);
                                            });
                                        };
                                        let on_b = move |_| {
                                            set_answers.update(|ans| {
                                                ans.push(false);
                                                push_hash_full(&items.get(), ans);
                                            });
                                        };

                                        view! {
                                            <section class="compare">
                                                <h2 class="compare-prompt">"Which do you prefer?"</h2>
                                                <div class="compare-buttons">
                                                    <button class="choice-btn" on:click=on_a>
                                                        {items_inner[a].clone()}
                                                    </button>
                                                    <span class="vs">"vs"</span>
                                                    <button class="choice-btn" on:click=on_b>
                                                        {items_inner[b].clone()}
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
                                {items_for_tags
                                    .iter()
                                    .map(|name| {
                                        view! { <span class="item-tag">{name.clone()}</span> }
                                    })
                                    .collect_view()}
                            </div>
                        </section>
                    }
                    .into_any()
                }
            }}
        </main>
    }
}

#[component]
fn InputForm(
    set_items: WriteSignal<Vec<String>>,
    set_answers: WriteSignal<Vec<bool>>,
) -> impl IntoView {
    let (text, set_text) = signal(String::new());

    let on_start = move |_| {
        let raw = text.get();
        let new_items: Vec<String> = raw
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        if new_items.len() >= 2 {
            push_hash_full(&new_items, &[]);
            set_answers.set(Vec::new());
            set_items.set(new_items);
        }
    };

    let item_count =
        Memo::new(move |_| text.get().lines().filter(|l| !l.trim().is_empty()).count());

    view! {
        <section class="input-form">
            <h2 class="input-title">"Enter items to rank"</h2>
            <p class="input-hint">"One item per line (minimum 2)"</p>
            <textarea
                class="item-textarea"
                rows="8"
                placeholder="Pizza\nSushi\nTacos\n..."
                prop:value=move || text.get()
                on:input=move |ev| {
                    set_text.set(event_target_value(&ev));
                }
            />
            <button
                class="start-btn"
                on:click=on_start
                disabled=move || item_count.get() < 2
            >
                {move || {
                    let count = item_count.get();
                    if count < 2 {
                        "Enter at least 2 items".to_string()
                    } else {
                        format!("Start ranking ({count} items)")
                    }
                }}
            </button>
        </section>
    }
}
