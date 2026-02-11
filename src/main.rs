use std::io::{self, Write};

use rankfast::rank_items;

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

    let ranking = rank_items(items, |a, b| compare(a, b));

    println!("Final ranking:");
    if ranking.is_empty() {
        println!("(empty)");
        return;
    }
    for (i, name) in ranking.iter().enumerate() {
        println!("{}. {}", i + 1, name);
    }
}

fn compare(a: &str, b: &str) -> bool {
    loop {
        print!("Which is better? Type A or B: [{a}] vs [{b}] ");
        io::stdout().flush().ok();

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
