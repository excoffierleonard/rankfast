/// Sorts `items` using the Ford-Johnson merge-insertion algorithm,
/// which is designed to minimize the number of calls to `better`.
///
/// # Comparator contract
///
/// `better(a, b)` must define a strict weak ordering over the items
/// (irreflexive, transitive, and consistent). Results are undefined if
/// this contract is violated.
///
/// # Panics
///
/// Cannot panic. The internal `expect` is guarded by construction.
#[must_use]
pub fn rank_items<T, F>(items: Vec<T>, mut better: F) -> Vec<T>
where
    F: FnMut(&T, &T) -> bool,
{
    let n = items.len();
    if n <= 1 {
        return items;
    }

    let indices: Vec<usize> = (0..n).collect();
    let sorted = ford_johnson(indices, &mut |a, b| better(&items[a], &items[b]));

    let mut slots: Vec<Option<T>> = items.into_iter().map(Some).collect();
    sorted
        .into_iter()
        .map(|i| slots[i].take().expect("each index used exactly once"))
        .collect()
}

/// Returns an upper-bound estimate of the number of comparisons (turns)
/// `rank_items` may need for `n` items.
///
/// The estimate assumes worst-case paths in binary searches. Actual turns
/// can be lower depending on the comparator outcomes.
#[must_use]
pub fn estimate_turns(n: usize) -> usize {
    if n <= 1 {
        return 0;
    }

    let num_pairs = n / 2;
    let mut total = num_pairs + estimate_turns(num_pairs);

    // After the initial chain is built, we insert the remaining elements.
    // Each insertion performs a binary search over a prefix of the chain.
    // We use an upper bound where the prefix is as large as possible.
    for chain_len in (num_pairs + 1)..n {
        total += ceil_log2(chain_len + 1);
    }

    total
}

/// Sorts a vec of element IDs using Ford-Johnson.
/// `cmp(a, b)` returns true when `a` should rank before `b`.
fn ford_johnson(elements: Vec<usize>, cmp: &mut impl FnMut(usize, usize) -> bool) -> Vec<usize> {
    let n = elements.len();
    if n <= 1 {
        return elements;
    }

    // Step 1: Pair up and compare. The worse element of each pair ("main")
    // goes into the recursive step; the better element ("partner") gets a
    // free insertion later because partner < main.
    let num_pairs = n / 2;
    let max_elem = elements.iter().copied().max().unwrap_or(0);
    let mut mains = Vec::with_capacity(num_pairs);
    let mut partner_of = vec![0usize; max_elem + 1];

    for i in 0..num_pairs {
        let (a, b) = (elements[2 * i], elements[2 * i + 1]);
        if cmp(a, b) {
            mains.push(b);
            partner_of[b] = a;
        } else {
            mains.push(a);
            partner_of[a] = b;
        }
    }
    let straggler = if n % 2 == 1 {
        Some(elements[n - 1])
    } else {
        None
    };

    // Step 2: Recursively sort the main (worse) elements.
    let sorted_mains = ford_johnson(mains, cmp);

    // Step 3: Build initial chain.
    // partner[sorted_mains[0]] is better than sorted_mains[0], which is better
    // than sorted_mains[1], etc. So the partner goes at the front for free.
    let mut chain = Vec::with_capacity(n);
    chain.push(partner_of[sorted_mains[0]]);
    chain.extend_from_slice(&sorted_mains);

    // Step 4: Collect remaining partners (and straggler) for insertion.
    // Each partner is better than its main, so we only search before the
    // main's current position in the chain.
    let mut pending: Vec<(usize, Option<usize>)> = Vec::new();
    for &m in sorted_mains.iter().skip(1) {
        pending.push((partner_of[m], Some(m)));
    }
    if let Some(s) = straggler {
        pending.push((s, None));
    }

    // Step 5: Insert in Jacobsthal order so each binary search operates on
    // a range of size 2^k - 1, wasting zero information per comparison.
    for i in jacobsthal_order(pending.len()) {
        let (elem, main) = pending[i];
        let bound = match main {
            Some(m) => chain.iter().position(|&x| x == m).unwrap(),
            None => chain.len(),
        };
        let pos = binary_search_pos(&chain[..bound], elem, cmp);
        chain.insert(pos, elem);
    }

    chain
}

fn ceil_log2(value: usize) -> usize {
    if value <= 1 {
        return 0;
    }
    let mut v = value - 1;
    let mut bits = 0usize;
    while v > 0 {
        bits += 1;
        v >>= 1;
    }
    bits
}

fn binary_search_pos(
    range: &[usize],
    element: usize,
    cmp: &mut impl FnMut(usize, usize) -> bool,
) -> usize {
    let (mut lo, mut hi) = (0, range.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if cmp(element, range[mid]) {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    lo
}

/// Returns indices into a `pending` array of length `count`, ordered by
/// Jacobsthal numbers for optimal insertion.
fn jacobsthal_order(count: usize) -> Vec<usize> {
    if count == 0 {
        return Vec::new();
    }
    // Jacobsthal boundaries (b-notation, 1-indexed): 1, 3, 5, 11, 21, 43, ...
    // Each group inserts from boundary[k] down to boundary[k-1]+1.
    // pending[i] corresponds to b_{i+2}, so b_k maps to index k-2.
    let mut order = Vec::with_capacity(count);
    let (mut prev, mut curr) = (1usize, 3usize);
    loop {
        let top = curr.min(count + 1);
        for b in (prev + 1..=top).rev() {
            order.push(b - 2);
        }
        if order.len() >= count {
            break;
        }
        let next = curr + 2 * prev;
        prev = curr;
        curr = next;
    }
    order
}

#[cfg(test)]
mod tests {
    use super::rank_items;

    #[test]
    fn ranks_numbers_ascending() {
        let items = vec![5, 2, 9, 1, 3];
        let ranked = rank_items(items, |a, b| a < b);
        assert_eq!(ranked, vec![1, 2, 3, 5, 9]);
    }

    #[test]
    fn ranks_strings_by_length_then_alpha() {
        let items = vec!["bbb", "a", "cc", "aa", "c"];
        let ranked = rank_items(items, |a, b| {
            a.len() < b.len() || (a.len() == b.len() && a < b)
        });
        assert_eq!(ranked, vec!["a", "c", "aa", "cc", "bbb"]);
    }

    #[test]
    fn worst_case_comparisons_are_optimal() {
        let optimal = [0, 0, 1, 3, 5, 7, 10, 13, 16];
        for (n, &opt) in optimal.iter().enumerate() {
            let mut worst = 0usize;
            let mut items: Vec<usize> = (0..n).collect();
            permute(&mut items, n, &mut |perm| {
                let mut count = 0usize;
                let ranked = rank_items(perm.to_vec(), |a, b| {
                    count += 1;
                    a < b
                });
                for w in ranked.windows(2) {
                    assert!(w[0] < w[1]);
                }
                worst = worst.max(count);
            });
            assert_eq!(worst, opt, "n={n}: worst={worst}, optimal={opt}");
        }
    }

    fn permute(items: &mut [usize], k: usize, f: &mut impl FnMut(&[usize])) {
        if k <= 1 {
            f(items);
            return;
        }
        permute(items, k - 1, f);
        for i in 0..k - 1 {
            items.swap(if k.is_multiple_of(2) { i } else { 0 }, k - 1);
            permute(items, k - 1, f);
        }
    }

    #[test]
    fn show_min_max_comparisons() {
        for n in 2..=8 {
            let (mut lo, mut hi) = (usize::MAX, 0usize);
            let mut items: Vec<usize> = (0..n).collect();
            permute(&mut items, n, &mut |perm| {
                let mut count = 0usize;
                let _ = rank_items(perm.to_vec(), |a, b| {
                    count += 1;
                    a < b
                });
                lo = lo.min(count);
                hi = hi.max(count);
            });
            println!("n={n}: min={lo} max={hi}");
        }
    }
}
