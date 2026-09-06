use engrams::secrets::scan_text;

#[test]
fn real_looking_secrets_are_found_with_their_line() {
    let text = "title\nglpat-ABCDEFGHIJKLMNOPQR\nnothing\nsk-ant-abcdefghijklmnopqrstuvwxyz\npassword = Tr0ub4dor&3xx\n";
    let hits = scan_text("a.md", text);
    let kinds: Vec<(&str, usize)> = hits.iter().map(|h| (h.kind, h.line)).collect();
    assert!(kinds.contains(&("GitLab token", 2)), "{kinds:?}");
    assert!(kinds.contains(&("Anthropic key", 4)), "{kinds:?}");
    assert!(kinds.contains(&("password", 5)), "{kinds:?}");
}

#[test]
fn documented_placeholders_are_not_secrets() {
    let text = "grant_type=password&username=x\npassword: <fill in>\nmdp = ...\npassword=MY_PASSWORD_HERE\nthe word passwd: short\n10 POST `password=WRONG` then locked\n";
    assert!(scan_text("a.md", text).is_empty());
}
