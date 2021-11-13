use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::configuration_reader::api_def_reader::APIDefinition;
use crate::configuration_reader::origin_def_reader::{Origin, OriginSpecification};
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

pub(crate) fn read_all_api_definitions() -> Vec<APIDefinition> {
    let mut api_definitions = vec![];
    let all_file_paths = read_config_file_paths(
        get_directory_of_executable().join(Path::new("/config/definitions/api_def")),
    );
    for path_buffer in all_file_paths {
        match FileReader::from_path(path_buffer.to_str().unwrap()).read() {
            Ok(json_payload) => {
                let api_def_read_result = APIDefinition::from_json_string(&json_payload);
                match api_def_read_result {
                    Ok(api_definition) => {
                        api_definitions.push(api_definition);
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to parse JSON content in file {} as APIDefinition because {}",
                            path_buffer.to_str().unwrap(),
                            e
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to read file at {} because {}!",
                    path_buffer.to_str().unwrap(),
                    e.message
                );
            }
        }
    }
    api_definitions
}

pub(crate) fn read_all_origin_definitions() -> Vec<Origin> {
    let mut origin_definitions = vec![];
    let all_file_paths = read_config_file_paths(
        get_directory_of_executable().join(Path::new("/config/definitions/origin_def")),
    );
    for path_buffer in all_file_paths {
        match FileReader::from_path(path_buffer.to_str().unwrap()).read() {
            Ok(json_payload) => {
                let origin_def_read_result = Origin::from_json_string(&json_payload);
                match origin_def_read_result {
                    Ok(origin_definition) => {
                        origin_definitions.push(origin_definition);
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to parse JSON content in file {} as Origin because {}",
                            path_buffer.to_str().unwrap(),
                            e
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to read file at {} because {}!",
                    path_buffer.to_str().unwrap(),
                    e.message
                );
            }
        }
    }
    origin_definitions
}
