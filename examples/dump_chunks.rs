//! Exports the chunks exactly as the index embeds them (same split, same prefix),
//! so another engine can be measured on the same material.
use engrams::chunking::{budget_for, split};
use engrams::note::Note;

fn main() {
    let root = engrams::paths::root();
    let window: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(512);
    let budget = budget_for(window);
    let mut rows = Vec::new();
    for f in engrams::hot::notes_of(&root) {
        let Ok(content) = std::fs::read_to_string(&f) else { continue };
        let path = engrams::paths::relative(&f, &root);
        let note = Note::parse(&content);
        let (name, desc) = (note.field("name").unwrap_or_default(), note.field("description").unwrap_or_default());
        for c in split(note.body(), budget) {
            rows.push(serde_json::json!({
                "key": format!("{path}#{}", c.ordinal),
                "path": path,
                "ordinal": c.ordinal,
                "active": note.is_active(),
                "text": c.with_context(name, desc),
            }));
        }
    }
    println!("{}", serde_json::Value::Array(rows));
}
