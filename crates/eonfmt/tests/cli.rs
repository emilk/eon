use std::{
    fs,
    io::Write as _,
    path::PathBuf,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("eonfmt-{label}-{}-{unique}", std::process::id()))
}

#[test]
fn formats_stdin() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_eonfmt"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn eonfmt");

    child
        .stdin
        .as_mut()
        .expect("missing stdin")
        .write_all(b"key:true//comment\n")
        .expect("write stdin");

    let output = child.wait_with_output().expect("wait on eonfmt");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "key: true //comment\n"
    );
}

#[test]
fn check_mode_reports_unformatted_file() {
    let dir = temp_path("check");
    fs::create_dir_all(&dir).expect("create temp dir");

    let file = dir.join("config.eon");
    fs::write(&file, "key:true\n").expect("write temp file");

    let output = Command::new(env!("CARGO_BIN_EXE_eonfmt"))
        .arg("--check")
        .arg(&file)
        .output()
        .expect("run eonfmt --check");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Would format"));

    fs::remove_dir_all(&dir).expect("remove temp dir");
}

#[test]
fn formats_each_matching_file_in_directory() {
    let dir = temp_path("walk");
    fs::create_dir_all(&dir).expect("create temp dir");

    let eon_file = dir.join("a.eon");
    let txt_file = dir.join("b.txt");
    fs::write(&eon_file, "key:true\n").expect("write eon file");
    fs::write(&txt_file, "leave me alone\n").expect("write txt file");

    let output = Command::new(env!("CARGO_BIN_EXE_eonfmt"))
        .arg(&dir)
        .output()
        .expect("run eonfmt on directory");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(&eon_file).expect("read eon file"),
        "key: true\n"
    );
    assert_eq!(
        fs::read_to_string(&txt_file).expect("read txt file"),
        "leave me alone\n"
    );

    fs::remove_dir_all(&dir).expect("remove temp dir");
}
