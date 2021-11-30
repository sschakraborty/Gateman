use std::env;
use std::path::PathBuf;

pub fn get_directory_of_executable() -> PathBuf {
    let mut executable_path = env::current_exe().unwrap();
    assert_eq!(true, executable_path.pop());
    executable_path
}
