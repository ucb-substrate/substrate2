use std::io::Read;
use std::process::Command;

use substrate::execute::{ExecOpts, Executor, LsfExecutor};

use crate::paths::get_path;

#[test]
#[ignore = "flaky"]
fn can_submit_with_bsub() {
    let file = get_path("can_submit_with_bsub", "file.txt");

    // Ignore errors here (it is ok if the directory does not exist).
    let _ = std::fs::remove_dir_all(file.parent().unwrap());

    std::fs::create_dir_all(file.parent().unwrap()).unwrap();

    let mut cmd = Command::new("bash");
    cmd.arg("-c")
        .arg(format!("echo \"Hello, world!\" > {file:?}"));

    let bsub = LsfExecutor::builder().queue("bora").build().unwrap();
    bsub.execute(cmd, Default::default()).expect("bsub failed");

    // Wait for filesystem to sync.
    std::thread::sleep(std::time::Duration::from_secs(3));

    let mut file = std::fs::File::open(&file).unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    assert_eq!(buf.trim(), "Hello, world!");
}

#[test]
fn lsf_executor_command() {
    let mut cmd = Command::new("touch");
    cmd.arg("hello.txt");

    let exec = LsfExecutor::builder()
        .bsub("mysub")
        .queue("myqueue")
        .build()
        .unwrap();
    let submit = exec.command(
        &cmd,
        ExecOpts {
            cpus: Some(2),
            ..Default::default()
        },
    );

    assert_eq!(submit.get_program(), "mysub");
    let args = submit.get_args().collect::<Vec<_>>();
    assert_eq!(args[0], "-K");
    assert_eq!(args[1], "-q");
    assert_eq!(args[2], "myqueue");
    assert_eq!(args[3], "-n");
    assert_eq!(args[4], "2");
    assert_eq!(args[5], "touch");
    assert_eq!(args[6], "hello.txt");
}
