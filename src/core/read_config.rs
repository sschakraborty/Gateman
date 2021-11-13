use std::env;
use std::fs;
use std::path::PathBuf;

use crate::file_utils::file_reader::FileReader;

fn get_directory_of_executable() -> PathBuf {
    env::current_dir().unwrap()
}

fn read_config_file_paths(current_directory: PathBuf) -> Vec<PathBuf> {
    let dir_traversal_result = fs::read_dir(current_directory.clone());
    let mut file_list = vec![];
    match dir_traversal_result {
        Ok(directory_contents) => {
            for element_result in directory_contents {
                match element_result {
                    Ok(element) => {
                        if element.file_type().unwrap().is_dir() {
                            file_list.append(&mut read_config_file_paths(element.path()));
                        } else if element.file_type().unwrap().is_file()
                            && element.file_name().to_str().unwrap().contains(".json")
                        {
                            file_list.push(element.path());
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read file {}!", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!(
                "Failed to read the directory at {} because {}!",
                current_directory.to_str().unwrap(),
                e
            );
        }
    }
    file_list
}

pub fn read_all_config_files() {
    let all_file_paths = read_config_file_paths(get_directory_of_executable());
    for path_buffer in all_file_paths {
        match FileReader::from_path(path_buffer.to_str().unwrap()).read() {
            Ok(_json_payload) => {}
            Err(e) => {
                eprintln!(
                    "Failed to read file at {} because {}!",
                    path_buffer.to_str().unwrap(),
                    e.message
                );
            }
        }
    }
}
