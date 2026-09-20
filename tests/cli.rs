use std::process::Command;
use tempfile::tempdir;

fn run(tsv: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_idid"))
        .arg("--tsv")
        .arg(tsv)
        .args(args)
        .output()
        .expect("failed to run idid")
}

#[test]
fn records_and_reports_an_adjusted_entry() {
    let directory = tempdir().unwrap();
    let tsv = directory.path().join("idid.tsv");
    std::fs::File::create(&tsv).unwrap();

    let start = run(&tsv, &["start", "-t", "30", "--quiet"]);
    assert!(
        start.status.success(),
        "{}",
        String::from_utf8_lossy(&start.stderr)
    );

    let add = run(&tsv, &["add", "-t", "20", "--quiet", "reviewed", "notes"]);
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );

    let show = run(&tsv, &["show", "today", "--json"]);
    assert!(
        show.status.success(),
        "{}",
        String::from_utf8_lossy(&show.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(json["duration"], "00:10");
    assert_eq!(json["text"], "reviewed notes");

    let last = run(&tsv, &["last", "2"]);
    assert!(
        last.status.success(),
        "{}",
        String::from_utf8_lossy(&last.stderr)
    );
    let output = String::from_utf8_lossy(&last.stdout);
    assert!(output.contains("reviewed notes"));
    assert!(output.contains("*~*~*--------------------"));
}

#[test]
fn creates_the_default_xdg_directory() {
    let directory = tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_idid"))
        .env("XDG_DATA_HOME", directory.path())
        .env_remove("ididTSV")
        .args(["start", "--quiet"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(directory.path().join("idid/idid.tsv").is_file());
}
