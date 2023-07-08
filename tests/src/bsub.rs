use std::process::Command;

use substrate::execute::{Executor, LsfExecutor};

use crate::paths::get_path;

#[test]
fn can_submit_with_bsub() {
    let file = get_path("can_submit_with_bsub", "file.txt");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();

    // Ignore errors here (it is ok if the file does not exist).
    let _ = std::fs::remove_file(&file);
    assert!(!file.exists());

    let mut cmd = Command::new("touch");
    cmd.arg(&file);

    let bsub = LsfExecutor::default();
    bsub.execute(cmd, Default::default()).expect("bsub failed");

    assert!(file.exists());
}
