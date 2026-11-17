//! Launcher search.
//!
//! A simple fuzzy/substring search over the application list. Real
//! implementations may add ranking, recent-first, and prefix matching.

use crate::apps::AppEntry;

/// Score an app against a query. Higher is better. Returns None if no match.
pub fn score(app: &AppEntry, query: &str) -> Option<i32> {
    let q = query.to_lowercase();
    if q.is_empty() {
        return Some(0);
    }

    let name = app.name.to_lowercase();

    // Exact name match.
    if name == q {
        return Some(100);
    }
    // Name starts with query.
    if name.starts_with(&q) {
        return Some(80);
    }
    // Name contains query.
    if name.contains(&q) {
        return Some(60);
    }
    // Exec contains query.
    if app.exec.to_lowercase().contains(&q) {
        return Some(30);
    }
    // Category contains query.
    if app.categories.iter().any(|c| c.to_lowercase().contains(&q)) {
        return Some(20);
    }

    None
}

/// Filter and rank applications by a query.
pub fn search<'a>(apps: &'a [AppEntry], query: &str) -> Vec<&'a AppEntry> {
    let mut scored: Vec<(i32, &AppEntry)> = apps
        .iter()
        .filter_map(|app| score(app, query).map(|s| (s, app)))
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().map(|(_, app)| app).collect()
}
