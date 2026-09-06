//! Life-cycle commands, end to end, on a temporary root: the compiled binary, a git
//! repository, two notes. No model: the duplicate check is skipped when the weights
//! are absent, which is the intended case here.
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

struct Root(PathBuf);

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn root(name: &str) -> Root {
    let r = std::env::temp_dir().join(format!("souvenance-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&r);
    std::fs::create_dir_all(r.join("fam/proj")).unwrap();
    std::fs::write(
        r.join("fam/proj/alpha.md"),
        "---\nname: alpha\ndescription: the alpha note\ntype: project\nstatus: active\nverified: 2026-01-01\n---\n\nBody of alpha, see [[beta]].\n",
    )
    .unwrap();
    std::fs::write(
        r.join("fam/proj/beta.md"),
        "---\nname: beta\ndescription: the beta note\ntype: reference\nstatus: active\nverified: 2026-01-01\n---\n\nBody of beta.\n",
    )
    .unwrap();
    for args in [vec!["init", "-q"], vec!["add", "-A"], vec!["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "init"]] {
        Command::new("git").arg("-C").arg(&r).args(&args).status().unwrap();
    }
    Root(r)
}

fn souvenance(root: &Path, args: &[&str], stdin: &str) -> (bool, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_souvenance"))
        .args(args)
        .env("SOUVENANCE_ROOT", root)
        .env("SOUVENANCE_MODEL", root.join("no-model"))
        .env("SOUVENANCE_NO_DAEMON", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    (out.status.success(), String::from_utf8_lossy(&out.stdout).into_owned(), String::from_utf8_lossy(&out.stderr).into_owned())
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

#[test]
fn write_creates_a_conforming_note_and_refuses_secrets_and_collisions() {
    let r = root("write");
    let (ok, out, _) = souvenance(&r.0, &["write", "fam/proj", "gamma", "--type", "feedback", "--description", "rule: never do X"], "The body of gamma.\n");
    assert!(ok, "{out}");
    let g = read(&r.0, "fam/proj/gamma.md");
    assert!(g.starts_with("---\nname: gamma\ndescription: \"rule: never do X\"\ntype: feedback\nstatus: active\nverified: "), "{g}");
    assert!(read(&r.0, "fam/proj/MEMORY.md").contains("[gamma](gamma.md)"), "the hot index follows the write");
    let (ok, _, err) = souvenance(&r.0, &["write", "fam/proj", "gamma", "--type", "feedback", "--description", "d"], "again");
    assert!(!ok && err.contains("already exists"), "{err}");
    let (ok, _, err) = souvenance(&r.0, &["write", "fam/proj", "delta", "--type", "project", "--description", "d"], "token glpat-ABCDEFGHIJKLMNOPQRS here");
    assert!(!ok && err.contains("secret"), "{err}");
    assert!(!r.0.join("fam/proj/delta.md").exists());
    let (ok, _, err) = souvenance(&r.0, &["write", "fam/proj", "Bad_Name", "--type", "project", "--description", "d"], "body");
    assert!(!ok && err.contains("convention"), "{err}");
}

#[test]
fn append_and_verify_update_the_note_and_its_date() {
    let r = root("append");
    let (ok, out, _) = souvenance(&r.0, &["append", "alpha"], "One more fact.");
    assert!(ok, "{out}");
    let a = read(&r.0, "fam/proj/alpha.md");
    assert!(a.ends_with("see [[beta]].\n\nOne more fact.\n"), "{a}");
    assert!(!a.contains("verified: 2026-01-01"), "verified must be today");
    let (ok, _, _) = souvenance(&r.0, &["verify", "beta"], "");
    assert!(ok);
    assert!(!read(&r.0, "fam/proj/beta.md").contains("verified: 2026-01-01"));
    let (ok, _, err) = souvenance(&r.0, &["verify", "unknown"], "");
    assert!(!ok && err.contains("not found"));
}

#[test]
fn supersede_archives_the_old_note_and_rewrites_links_to_it() {
    let r = root("supersede");
    let (ok, out, err) = souvenance(&r.0, &["supersede", "beta", "beta-2", "--type", "reference", "--description", "the beta note, version 2"], "New body.");
    assert!(ok, "{out}{err}");
    let old = read(&r.0, "fam/proj/beta.md");
    assert!(old.contains("status: archived") && old.contains("superseded_by: [[beta-2]]"), "{old}");
    assert!(read(&r.0, "fam/proj/beta-2.md").contains("New body."));
    assert!(read(&r.0, "fam/proj/alpha.md").contains("see [[beta-2]]."), "alpha's link follows the replacement");
    let hot = read(&r.0, "fam/proj/MEMORY.md");
    assert!(hot.contains("[beta-2](beta-2.md)") && !hot.contains("[beta](beta.md)"), "{hot}");
    assert!(hot.contains("1 archived note(s)"));
}

#[test]
fn link_adds_a_cross_reference_both_ways_once() {
    let r = root("link");
    let (ok, _, err) = souvenance(&r.0, &["link", "alpha", "beta"], "");
    assert!(ok, "{err}");
    assert!(read(&r.0, "fam/proj/beta.md").contains("See also [[alpha]]"));
    // alpha already cited beta: no second reference
    assert_eq!(read(&r.0, "fam/proj/alpha.md").matches("[[beta]]").count(), 1);
    let (ok, _, _) = souvenance(&r.0, &["link", "alpha", "beta"], "");
    assert!(ok);
    assert_eq!(read(&r.0, "fam/proj/beta.md").matches("[[alpha]]").count(), 1, "idempotent");
}

#[test]
fn since_and_why_read_git_history() {
    let r = root("git");
    souvenance(&r.0, &["append", "alpha"], "Tracked addition.");
    Command::new("git").arg("-C").arg(&r.0).args(["add", "-A"]).status().unwrap();
    Command::new("git").arg("-C").arg(&r.0).args(["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "addition"]).status().unwrap();
    let (ok, out, _) = souvenance(&r.0, &["since", "1"], "");
    assert!(ok && out.contains("fam/proj") && out.contains("modified") && out.contains("alpha.md"), "{out}");
    let (ok, out, _) = souvenance(&r.0, &["why", "beta"], "");
    assert!(ok && out.contains("cited by 1 note(s)") && out.contains("alpha.md") && out.contains("git history"), "{out}");
}

#[test]
fn check_and_secrets_pass_on_a_clean_root() {
    let r = root("check");
    let (ok, out, _) = souvenance(&r.0, &["check"], "");
    assert!(ok, "{out}");
    let (ok, _, _) = souvenance(&r.0, &["secrets"], "");
    assert!(ok);
}

#[test]
fn learn_derives_missed_reads_and_records_explicit_pairs() {
    let r = root("learn");
    // A search showed alpha; beta was read right after: beta was missed. A read of
    // alpha (already shown) teaches nothing, nor does a read outside the window.
    std::fs::create_dir_all(r.0.join(".souvenance")).unwrap();
    std::fs::write(r.0.join(".souvenance/searches.log"), "1000\thow does beta work\tfam/proj/alpha.md\n").unwrap();
    std::fs::write(r.0.join(".souvenance/reads.log"), "1010\tfam/proj/beta.md\n1020\tfam/proj/alpha.md\n5000\tfam/proj/beta.md\n").unwrap();
    let (ok, out, err) = souvenance(&r.0, &["learn"], "");
    assert!(ok, "{err}");
    assert!(out.starts_with("1 pair(s) learned"), "{out}");
    let table = std::fs::read_to_string(r.0.join(".souvenance/feedback.json")).unwrap();
    assert!(table.contains("\"query\": \"how does beta work\"") && table.contains("fam/proj/beta.md") && table.contains("\"missed\""), "{table}");
    assert!(!table.contains("alpha.md"), "a note already shown is not feedback");
    let (_, out, _) = souvenance(&r.0, &["learn"], "");
    assert!(out.starts_with("0 pair(s)"), "{out}");

    let (ok, out, err) = souvenance(&r.0, &["learn", "where is alpha", "alpha"], "");
    assert!(ok && out.contains("learned: \"where is alpha\" -> fam/proj/alpha.md"), "{out}{err}");
    let (_, out, _) = souvenance(&r.0, &["learn", "--show"], "");
    assert!(out.starts_with("2 learned pair(s)") && out.contains("explicit") && out.contains("missed"), "{out}");
    let (_, out, _) = souvenance(&r.0, &["learn", "--forget", "where is alpha"], "");
    assert!(out.starts_with("1 pair(s) forgotten"), "{out}");
    let (_, out, _) = souvenance(&r.0, &["learn", "--show"], "");
    assert!(out.starts_with("1 learned pair(s)"), "{out}");

    // A read is logged for the next pass.
    let (ok, _, _) = souvenance(&r.0, &["read", "alpha"], "");
    assert!(ok);
    let reads = std::fs::read_to_string(r.0.join(".souvenance/reads.log")).unwrap();
    assert!(reads.lines().last().unwrap().ends_with("\tfam/proj/alpha.md"), "{reads}");
}

#[test]
fn the_mcp_server_lists_and_calls_tools_over_stdio() {
    let r = root("mcp");
    let script = concat!(
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n",
        "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\"}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"read\",\"arguments\":{\"name\":\"beta\"}}}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"append\",\"arguments\":{\"name\":\"beta\",\"text\":\"Added over MCP.\"}}}\n",
        "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"read\",\"arguments\":{\"name\":\"nope\"}}}\n",
    );
    let (ok, out, err) = souvenance(&r.0, &["mcp"], script);
    assert!(ok, "{err}");
    let lines: Vec<serde_json::Value> = out.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(lines.len(), 5, "{out}");
    assert_eq!(lines[0]["result"]["serverInfo"]["name"], "souvenance");
    let tools: Vec<&str> = lines[1]["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(tools, vec!["search", "answer", "read", "write", "append", "link", "learn"]);
    assert!(lines[2]["result"]["content"][0]["text"].as_str().unwrap().contains("Body of beta."));
    assert!(lines[3]["result"]["content"][0]["text"].as_str().unwrap().contains("completed"));
    assert!(read(&r.0, "fam/proj/beta.md").contains("Added over MCP."));
    assert_eq!(lines[4]["result"]["isError"], true);
}
