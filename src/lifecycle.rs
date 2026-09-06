//! The life cycle of a fact: complete, re-verify, replace, link. These operations
//! keep the rules without the caller having to know them. They edit the file text
//! and preserve everything they do not touch.

/// Sets or replaces a frontmatter field. The frontmatter must exist.
pub fn set_field(content: &str, key: &str, value: &str) -> Result<String, String> {
    let (head, body) = split_frontmatter(content)?;
    let mut lines: Vec<String> = head.lines().map(str::to_string).collect();
    let prefix = format!("{key}:");
    let value = quote_if_needed(value);
    match lines.iter_mut().find(|l| l.starts_with(&prefix)) {
        Some(line) => *line = format!("{key}: {value}"),
        None => lines.push(format!("{key}: {value}")),
    }
    Ok(format!("---\n{}\n---\n{body}", lines.join("\n")))
}

/// Removes a frontmatter field if present.
pub fn remove_field(content: &str, key: &str) -> Result<String, String> {
    let (head, body) = split_frontmatter(content)?;
    let prefix = format!("{key}:");
    let lines: Vec<&str> = head.lines().filter(|l| !l.starts_with(&prefix)).collect();
    Ok(format!("---\n{}\n---\n{body}", lines.join("\n")))
}

/// Appends a paragraph to the body, separated by a blank line.
pub fn append_paragraph(content: &str, paragraph: &str) -> String {
    let mut out = content.trim_end_matches('\n').to_string();
    out.push_str("\n\n");
    out.push_str(paragraph.trim());
    out.push('\n');
    out
}

/// Rewrites `[[old]]` into `[[new]]` for every spelling the link resolver accepts
/// (case, `_` for `-`; aliases and anchors kept). Returns the text and the count.
pub fn relink(content: &str, old_key: &str, new_name: &str) -> (String, usize) {
    let mut out = String::with_capacity(content.len());
    let mut rest = content;
    let mut n = 0;
    while let Some(start) = rest.find("[[") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("]]") else {
            out.push_str(&rest[start..]);
            return (out, n);
        };
        let inner = &after[..end];
        let (target, tail) = match inner.find(['|', '#']) {
            Some(i) => (&inner[..i], &inner[i..]),
            None => (inner, ""),
        };
        if crate::check::link_key(target.trim()) == old_key {
            out.push_str(&format!("[[{new_name}{tail}]]"));
            n += 1;
        } else {
            out.push_str(&format!("[[{inner}]]"));
        }
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    (out, n)
}

fn split_frontmatter(content: &str) -> Result<(&str, &str), String> {
    let rest = content.strip_prefix("---\n").ok_or("the note has no frontmatter")?;
    let end = rest.find("\n---").ok_or("unterminated frontmatter")?;
    let body = &rest[end + 4..];
    let body = body.strip_prefix('\n').unwrap_or(body);
    Ok((&rest[..end], body))
}

fn quote_if_needed(v: &str) -> String {
    if (v.contains(": ") || v.starts_with('[') || v.starts_with('#')) && !v.starts_with('"') && !v.starts_with("[[") {
        format!("\"{}\"", v.replace('"', "\\\""))
    } else {
        v.to_string()
    }
}
