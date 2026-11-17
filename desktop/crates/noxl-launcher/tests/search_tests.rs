//! Tests for launcher search ranking.

use noxl_launcher::apps::AppEntry;
use noxl_launcher::search;

fn app(name: &str, exec: &str, categories: &[&str]) -> AppEntry {
    AppEntry {
        id: format!("{}.desktop", name.to_lowercase()),
        name: name.into(),
        icon: String::new(),
        categories: categories.iter().map(|s| s.to_string()).collect(),
        exec: exec.into(),
        visible: true,
    }
}

#[test]
fn exact_match_ranks_highest() {
    let a = app("Firefox", "firefox", &["Network"]);
    let score = search::score(&a, "firefox").unwrap();
    assert_eq!(score, 100);
}

#[test]
fn prefix_match_ranks_second() {
    let a = app("Firefox Developer Edition", "firefox-developer", &["Network"]);
    let score = search::score(&a, "fire").unwrap();
    assert_eq!(score, 80);
}

#[test]
fn contains_match_ranks_third() {
    let a = app("Mozilla Firefox", "firefox", &["Network"]);
    let score = search::score(&a, "fire").unwrap();
    assert_eq!(score, 60);
}

#[test]
fn exec_match_ranks_lower() {
    let a = app("Web Browser", "firefox", &["Network"]);
    let score = search::score(&a, "firefox").unwrap();
    assert_eq!(score, 30);
}

#[test]
fn category_match_ranks_lowest() {
    let a = app("Some App", "someapp", &["Network", "WebBrowser"]);
    let score = search::score(&a, "network").unwrap();
    assert_eq!(score, 20);
}

#[test]
fn no_match_returns_none() {
    let a = app("Firefox", "firefox", &["Network"]);
    assert!(search::score(&a, "thunderbird").is_none());
}

#[test]
fn empty_query_matches_everything() {
    let a = app("Firefox", "firefox", &["Network"]);
    assert_eq!(search::score(&a, ""), Some(0));
}

#[test]
fn search_orders_by_score() {
    let apps = vec![
        app("Web Browser", "firefox", &["Network"]),
        app("Firefox", "firefox", &["Network"]),
        app("Firefox Developer", "firefox-dev", &["Network"]),
    ];
    let results = search::search(&apps, "firefox");
    assert_eq!(results.len(), 3);
    // Exact match first.
    assert_eq!(results[0].name, "Firefox");
}

#[test]
fn search_is_case_insensitive() {
    let a = app("Firefox", "firefox", &["Network"]);
    assert!(search::score(&a, "FIREFOX").is_some());
    assert!(search::score(&a, "FireFox").is_some());
}
