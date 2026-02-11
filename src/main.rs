use std::io::{self, Write};

fn main() {
    // Hardcoded items to rank.
    let items = vec![
        "Blue".to_string(),
        "Orange".to_string(),
        "Red".to_string(),
        "Black".to_string(),
        "Green".to_string(),
        "Yellow".to_string(),
        "Purple".to_string(),
        "White".to_string(),
    ];

    let mut ranking: Vec<String> = Vec::new();

    for item in items {
        let idx = binary_insert_index(&item, &ranking);
        ranking.insert(idx, item);
    }

    println!("Final ranking:");
    if ranking.is_empty() {
        println!("(empty)");
        return;
    }
    for (i, name) in ranking.iter().enumerate() {
        println!("{}. {}", i + 1, name);
    }
}

fn binary_insert_index(item: &str, ranking: &[String]) -> usize {
    let mut lo = 0usize;
    let mut hi = ranking.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if compare(item, &ranking[mid]) {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    lo
}

fn compare(a: &str, b: &str) -> bool {
    loop {
        print!("Which is better? Type A or B: [{}] vs [{}] ", a, b);
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Could not read input. Try again.");
            continue;
        }

        let answer = input.trim();
        if answer.eq_ignore_ascii_case("a") {
            return true;
        }
        if answer.eq_ignore_ascii_case("b") {
            return false;
        }

        println!("Please type A or B");
    }
}
