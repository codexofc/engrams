//! Splitting a note into indexable chunks.
//!
//! One vector for a long note represents its dominant topic, not its details, and
//! the model truncates what exceeds its window. So notes are split along their
//! markdown structure, never in the middle of a paragraph: a halved idea gives two
//! vectors that mean nothing.

/// A chunk of a note, with its position, so the passage that answered can be shown
/// and not only the note that contains it.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub ordinal: usize,
    pub start: usize,
    pub text: String,
}

impl Chunk {
    /// Prefixes the chunk with the note's name and description. A paragraph that
    /// never names its subject stays findable once it gets its title back.
    pub fn with_context(&self, name: &str, description: &str) -> String {
        let mut out = String::with_capacity(name.len() + description.len() + self.text.len() + 4);
        out.push_str(name);
        if !description.is_empty() {
            out.push('\n');
            out.push_str(description);
        }
        out.push_str("\n\n");
        out.push_str(&self.text);
        out
    }
}

/// Rough token estimate. The budget is a target, not a hard limit, and running the
/// tokenizer here would cost more than the split itself.
fn estimate_tokens(text: &str) -> usize {
    text.len() / 3 + 1
}

/// Splits a note body into chunks aiming at `budget` tokens.
pub fn split(body: &str, budget: usize) -> Vec<Chunk> {
    let blocks = blocks(body);
    let mut out: Vec<Chunk> = Vec::new();
    let mut current: Option<(usize, String)> = None;

    for (start, block) in blocks {
        let is_heading = block.trim_start().starts_with('#');
        let too_big = current.as_ref().is_some_and(|(_, acc)| estimate_tokens(acc) + estimate_tokens(block) > budget);
        // A heading marks a change of topic, so it always opens a chunk.
        if is_heading || too_big {
            if let Some((s, text)) = current.take() {
                push(&mut out, s, text);
            }
        }
        match &mut current {
            Some((_, acc)) => {
                acc.push_str("\n\n");
                acc.push_str(block);
            }
            None => current = Some((start, block.to_string())),
        }
    }
    if let Some((s, text)) = current.take() {
        push(&mut out, s, text);
    }
    out
}

fn push(out: &mut Vec<Chunk>, start: usize, text: String) {
    if text.trim().is_empty() {
        return;
    }
    out.push(Chunk { ordinal: out.len(), start, text });
}

/// Blocks separated by a blank line, with their byte offset. A block is never cut,
/// even when it alone exceeds the budget.
fn blocks(body: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let bytes = body.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\n' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
            let block = body[start..i].trim();
            if !block.is_empty() {
                out.push((start + leading_ws(&body[start..i]), block));
            }
            i += 2;
            while i < bytes.len() && bytes[i] == b'\n' {
                i += 1;
            }
            start = i;
            continue;
        }
        i += 1;
    }
    let block = body[start..].trim();
    if !block.is_empty() {
        out.push((start + leading_ws(&body[start..]), block));
    }
    out
}

fn leading_ws(s: &str) -> usize {
    s.len() - s.trim_start().len()
}

/// Room left for the context prefix and the sequence markers.
const CONTEXT_RESERVE: usize = 62;

/// Chunk capacity derived from the model window, so the split follows the model.
pub fn budget_for(window: usize) -> usize {
    window.saturating_sub(CONTEXT_RESERVE).max(1)
}
