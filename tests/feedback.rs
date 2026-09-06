//! Learning must learn nothing from the engine itself, stay bounded, and decay.
use engrams::feedback::{decay, learned_bonus, missed_pairs, within_window, Read, Searched, Table, CAP, STEP};
use std::collections::HashMap;

#[test]
fn a_read_of_a_note_the_search_did_not_show_is_a_missed_pair_and_a_shown_one_is_not() {
    let searches = vec![
        Searched { secs: 1000, query: "signature rule".into(), shown: vec!["a.md".into(), "b.md".into()] },
        Searched { secs: 5000, query: "other".into(), shown: vec!["z.md".into()] },
    ];
    let reads = vec![
        Read { secs: 1100, path: "c.md".into() }, // missed: learned
        Read { secs: 1200, path: "a.md".into() }, // shown: nothing to learn
        Read { secs: 4000, path: "d.md".into() }, // too long after the search
        Read { secs: 5100, path: "y.md".into() }, // missed for "other"
    ];
    let pairs = missed_pairs(&searches, &reads, 600);
    let got: Vec<(&str, &str)> = pairs.iter().map(|(q, p, _)| (q.as_str(), p.as_str())).collect();
    assert_eq!(got, vec![("signature rule", "c.md"), ("other", "y.md")]);
}

#[test]
fn recording_counts_once_per_day_and_explicit_wins() {
    let mut t = Table::default();
    assert!(t.record("q", "a.md", "missed", 10));
    assert!(!t.record("q", "a.md", "missed", 10), "same day, no second count");
    assert!(t.record("q", "a.md", "explicit", 11));
    assert_eq!(t.pairs[0].count, 2);
    assert_eq!(t.pairs[0].source, "explicit");
    assert_eq!(t.forget("q"), 1);
    assert!(t.pairs.is_empty());
}

#[test]
fn the_bonus_is_capped_decays_and_only_applies_to_similar_queries() {
    let mut t = Table::default();
    for i in 0..10 {
        t.record("q", "a.md", "explicit", i);
    }
    t.record("far", "b.md", "explicit", 9);
    let sim = |q: &str| Some(if q == "q" { 0.95 } else { 0.5 });
    let b = learned_bonus(&t, 9, sim, None);
    assert!(b.get("a.md").unwrap() <= &CAP);
    assert!(!b.contains_key("b.md"), "a past query too different brings nothing");
    // One fresh pair: STEP times the similarity times one third.
    let mut one = Table::default();
    one.record("q", "a.md", "missed", 100);
    let b = learned_bonus(&one, 100, sim, None);
    assert!((b["a.md"] - STEP * 0.95 / 3.0).abs() < 1e-6, "{b:?}");
    // Sixty days later, half.
    let b = learned_bonus(&one, 160, sim, None);
    assert!((b["a.md"] - STEP * 0.95 / 3.0 * 0.5).abs() < 1e-6, "{b:?}");
    assert!((decay(0, 60) - 0.5).abs() < 1e-6);
    // The benchmark leaves the evaluated pair out.
    assert!(learned_bonus(&one, 100, sim, Some("q")).is_empty());
}

#[test]
fn the_bonus_never_lifts_a_far_note() {
    let bonus: HashMap<String, f32> = [("near.md".to_string(), 0.05), ("far.md".to_string(), 0.05)].into_iter().collect();
    let base: HashMap<String, f32> = [("top.md".to_string(), 0.80), ("near.md".to_string(), 0.72), ("far.md".to_string(), 0.55)].into_iter().collect();
    let kept = within_window(bonus, &base);
    assert!(kept.contains_key("near.md") && !kept.contains_key("far.md"), "{kept:?}");
}
