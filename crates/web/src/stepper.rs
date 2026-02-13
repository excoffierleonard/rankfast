use rankfast::jacobsthal_order;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    Compare { a: usize, b: usize },
    Done,
}

pub(crate) struct Stepper {
    stack: Vec<Frame>,
    pending: Option<Pending>,
    comparisons: usize,
    done: Option<Vec<usize>>,
}

impl Stepper {
    pub(crate) fn new(n: usize) -> Self {
        if n <= 1 {
            return Self {
                stack: Vec::new(),
                pending: None,
                comparisons: 0,
                done: Some((0..n).collect()),
            };
        }

        Self {
            stack: vec![Frame::new((0..n).collect())],
            pending: None,
            comparisons: 0,
            done: None,
        }
    }

    /// Advances the sorter until it needs a comparison or is done.
    ///
    /// # Panics
    ///
    /// Panics if the internal state machine is inconsistent, which indicates
    /// a bug in the stepper implementation.
    pub(crate) fn step(&mut self) -> Step {
        if let Some(step) = self.pending_step() {
            return step;
        }

        loop {
            if self.done.is_some() {
                return Step::Done;
            }

            if self.stack.is_empty() {
                self.done = Some(Vec::new());
                return Step::Done;
            }

            if self.pop_done_frame() {
                continue;
            }

            if let Some(step) = self.advance_frame() {
                return step;
            }
        }
    }

    /// Applies the result of the last comparison and advances to the next step.
    ///
    /// # Panics
    ///
    /// Panics if the internal state machine is inconsistent.
    pub(crate) fn answer(&mut self, better_is_a: bool) -> Step {
        let Some(pending) = self.pending.take() else {
            return self.step();
        };

        self.comparisons += 1;

        match pending {
            Pending::Pairing { .. } => {
                let frame = self
                    .stack
                    .last_mut()
                    .expect("pairing answer requires active frame");
                let State::Pairing {
                    i,
                    mains,
                    partner_of,
                    ..
                } = &mut frame.state
                else {
                    unreachable!("pairing answer requires pairing state")
                };

                let a = frame.elements[2 * *i];
                let b = frame.elements[2 * *i + 1];
                if better_is_a {
                    mains.push(b);
                    partner_of[b] = a;
                } else {
                    mains.push(a);
                    partner_of[a] = b;
                }
                *i += 1;
            }
            Pending::Search { .. } => {
                let frame = self
                    .stack
                    .last_mut()
                    .expect("search answer requires active frame");
                let State::Insert {
                    chain,
                    order_idx,
                    search,
                    ..
                } = &mut frame.state
                else {
                    unreachable!("search answer requires insert state")
                };

                let search_state = search
                    .as_mut()
                    .expect("search state must exist for comparison");
                let mid = search_state.mid.take().expect("mid must be set");
                if better_is_a {
                    search_state.hi = mid;
                } else {
                    search_state.lo = mid + 1;
                }

                if search_state.lo == search_state.hi {
                    let pos = search_state.lo;
                    let elem = search_state.elem;
                    chain.insert(pos, elem);
                    *search = None;
                    *order_idx += 1;
                }
            }
        }

        self.step()
    }

    pub(crate) fn take_order(&mut self) -> Option<Vec<usize>> {
        self.done.take()
    }

    pub(crate) fn comparisons_made(&self) -> usize {
        self.comparisons
    }

    fn pending_step(&self) -> Option<Step> {
        let pending = self.pending?;
        match pending {
            Pending::Pairing { a, b } | Pending::Search { a, b } => Some(Step::Compare { a, b }),
        }
    }

    fn pop_done_frame(&mut self) -> bool {
        let is_done = matches!(
            self.stack.last().map(|frame| &frame.state),
            Some(State::Done(_))
        );
        if !is_done {
            return false;
        }

        let Some(frame) = self.stack.pop() else {
            return false;
        };
        let State::Done(result) = frame.state else {
            unreachable!("checked above")
        };
        self.propagate_result(result);
        true
    }

    fn advance_frame(&mut self) -> Option<Step> {
        let mut frame = self.stack.pop()?;
        let elements = &frame.elements;
        let state = std::mem::replace(&mut frame.state, State::Start);

        let (next_state, step, child) = match state {
            State::Start => (Self::advance_start(elements), None, None),
            State::Pairing {
                i,
                num_pairs,
                mains,
                partner_of,
                straggler,
            } => self.advance_pairing(elements, i, num_pairs, mains, partner_of, straggler),
            State::AwaitMains {
                partner_of,
                straggler,
            } => {
                frame.state = State::AwaitMains {
                    partner_of,
                    straggler,
                };
                self.stack.push(frame);
                unreachable!("awaiting child frame result")
            }
            State::Insert {
                chain,
                pending,
                order,
                order_idx,
                search,
            } => {
                let (state, step) = self.advance_insert(chain, pending, order, order_idx, search);
                (state, step, None)
            }
            State::Done(result) => (State::Done(result), None, None),
        };

        frame.state = next_state;
        self.stack.push(frame);
        if let Some(child) = child {
            self.stack.push(child);
        }
        step
    }

    fn advance_start(elements: &[usize]) -> State {
        let n = elements.len();
        if n <= 1 {
            return State::Done(elements.to_vec());
        }

        let num_pairs = n / 2;
        let max_elem = elements.iter().copied().max().unwrap_or(0);
        let partner_of = vec![0usize; max_elem + 1];
        let mains = Vec::with_capacity(num_pairs);
        let straggler = if n % 2 == 1 {
            Some(elements[n - 1])
        } else {
            None
        };

        State::Pairing {
            i: 0,
            num_pairs,
            mains,
            partner_of,
            straggler,
        }
    }

    fn advance_pairing(
        &mut self,
        elements: &[usize],
        i: usize,
        num_pairs: usize,
        mains: Vec<usize>,
        partner_of: Vec<usize>,
        straggler: Option<usize>,
    ) -> (State, Option<Step>, Option<Frame>) {
        if i < num_pairs {
            let a = elements[2 * i];
            let b = elements[2 * i + 1];
            self.pending = Some(Pending::Pairing { a, b });
            return (
                State::Pairing {
                    i,
                    num_pairs,
                    mains,
                    partner_of,
                    straggler,
                },
                Some(Step::Compare { a, b }),
                None,
            );
        }

        (
            State::AwaitMains {
                partner_of,
                straggler,
            },
            None,
            Some(Frame::new(mains)),
        )
    }

    fn advance_insert(
        &mut self,
        mut chain: Vec<usize>,
        pending: Vec<(usize, Option<usize>)>,
        order: Vec<usize>,
        mut order_idx: usize,
        mut search: Option<SearchState>,
    ) -> (State, Option<Step>) {
        if order_idx >= order.len() {
            return (State::Done(chain), None);
        }

        if search.is_none() {
            let idx = order[order_idx];
            let (elem, main) = pending[idx];
            let bound = match main {
                Some(m) => chain
                    .iter()
                    .position(|&x| x == m)
                    .expect("main must be in chain"),
                None => chain.len(),
            };
            search = Some(SearchState {
                elem,
                lo: 0,
                hi: bound,
                mid: None,
            });
        }

        let Some(search_state) = search.as_mut() else {
            return (
                State::Insert {
                    chain,
                    pending,
                    order,
                    order_idx,
                    search,
                },
                None,
            );
        };

        if search_state.lo == search_state.hi {
            let pos = search_state.lo;
            let elem = search_state.elem;
            chain.insert(pos, elem);
            search = None;
            order_idx += 1;
            return (
                State::Insert {
                    chain,
                    pending,
                    order,
                    order_idx,
                    search,
                },
                None,
            );
        }

        let mid = search_state.lo + (search_state.hi - search_state.lo) / 2;
        search_state.mid = Some(mid);
        let a = search_state.elem;
        let b = chain[mid];
        self.pending = Some(Pending::Search { a, b });
        (
            State::Insert {
                chain,
                pending,
                order,
                order_idx,
                search,
            },
            Some(Step::Compare { a, b }),
        )
    }

    fn propagate_result(&mut self, result: Vec<usize>) {
        let Some(parent) = self.stack.last_mut() else {
            self.done = Some(result);
            return;
        };

        let State::AwaitMains {
            partner_of,
            straggler,
        } = std::mem::replace(&mut parent.state, State::Start)
        else {
            unreachable!("only await-mains can receive a result")
        };

        let mut chain = Vec::with_capacity(parent.elements.len());
        chain.push(partner_of[result[0]]);
        chain.extend_from_slice(&result);

        let mut pending: Vec<(usize, Option<usize>)> = Vec::new();
        for &m in result.iter().skip(1) {
            pending.push((partner_of[m], Some(m)));
        }
        if let Some(s) = straggler {
            pending.push((s, None));
        }

        let order = jacobsthal_order(pending.len());
        parent.state = State::Insert {
            chain,
            pending,
            order,
            order_idx: 0,
            search: None,
        };
    }
}

#[derive(Debug)]
struct Frame {
    elements: Vec<usize>,
    state: State,
}

impl Frame {
    fn new(elements: Vec<usize>) -> Self {
        Self {
            elements,
            state: State::Start,
        }
    }
}

#[derive(Debug)]
enum State {
    Start,
    Pairing {
        i: usize,
        num_pairs: usize,
        mains: Vec<usize>,
        partner_of: Vec<usize>,
        straggler: Option<usize>,
    },
    AwaitMains {
        partner_of: Vec<usize>,
        straggler: Option<usize>,
    },
    Insert {
        chain: Vec<usize>,
        pending: Vec<(usize, Option<usize>)>,
        order: Vec<usize>,
        order_idx: usize,
        search: Option<SearchState>,
    },
    Done(Vec<usize>),
}

#[derive(Debug, Clone, Copy)]
struct SearchState {
    elem: usize,
    lo: usize,
    hi: usize,
    mid: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
enum Pending {
    Pairing { a: usize, b: usize },
    Search { a: usize, b: usize },
}
