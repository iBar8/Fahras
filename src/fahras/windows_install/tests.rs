// SPDX-License-Identifier: MIT

use super::*;

#[test]
fn an_installer_that_does_not_finish_is_reported_as_a_failure() {
    assert_eq!(run_installer(&mut Command::new("true")), Ok(()));
    let stopped = run_installer(&mut Command::new("false")).expect_err("a failed install");
    assert!(stopped.contains("(code 1)"), "{stopped}");
    let missing = run_installer(&mut Command::new("/nonexistent/Fahras-Setup.exe"))
        .expect_err("a missing installer");
    assert!(missing.contains("Could not start"), "{missing}");
}
