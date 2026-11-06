// SPDX-License-Identifier: MIT

use super::*;

#[test]
fn only_folders_of_ended_processes_count_as_left_behind() {
    let ended = {
        let mut child = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .arg("--list")
            .stdout(std::process::Stdio::null())
            .spawn()
            .expect("child process");
        let pid = child.id();
        child.wait().expect("child ends");
        pid
    };
    let folder = |name: String| std::env::temp_dir().join(name);
    assert!(owner_has_exited(&folder(format!("out-{ended}-1")), "out-"));
    assert!(!owner_has_exited(
        &folder(format!("out-{}-1", std::process::id())),
        "out-"
    ));
    assert!(!owner_has_exited(&folder("out-x-1".into()), "out-"));
    assert!(!owner_has_exited(
        &folder(format!("other-{ended}-1")),
        "out-"
    ));
}
