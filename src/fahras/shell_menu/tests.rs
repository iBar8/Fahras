// SPDX-License-Identifier: MIT

use super::*;

#[test]
fn the_shell_menu_is_only_for_files_in_one_folder() {
    let paths = |list: &[&str]| list.iter().map(PathBuf::from).collect::<Vec<_>>();
    assert!(share_one_folder(&paths(&["/a/x.txt", "/a/y.txt"])));
    assert!(share_one_folder(&paths(&["/a/x.txt"])));
    assert!(!share_one_folder(&paths(&["/a/x.txt", "/b/y.txt"])));
    assert!(!share_one_folder(&paths(&[])));
}
