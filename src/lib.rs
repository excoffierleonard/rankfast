pub fn rank_items<T, F>(items: Vec<T>, mut better: F) -> Vec<T>
where
    F: FnMut(&T, &T) -> bool,
{
    let mut ranking: Vec<T> = Vec::new();
    for item in items {
        let idx = binary_insert_index(&item, &ranking, &mut better);
        ranking.insert(idx, item);
    }
    ranking
}

fn binary_insert_index<T, F>(item: &T, ranking: &[T], better: &mut F) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    let mut lo = 0usize;
    let mut hi = ranking.len();
    while lo < hi {
        let mid = usize::midpoint(lo, hi);
        if better(item, &ranking[mid]) {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    lo
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
}
