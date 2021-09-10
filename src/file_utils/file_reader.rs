use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::file_utils::file_error::FileOperationError;

pub(crate) struct FileReader {
    filepath: String,
}

impl FileReader {
    pub(crate) fn from_path(path: &str) -> Self {
        FileReader {
            filepath: String::from(path),
        }
    }
    pub(crate) fn read(&self) -> Result<String, FileOperationError> {
        match File::open(Path::new(self.filepath.as_str())) {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content);
                Result::Ok(content)
            }
            Err(reason) => Result::Err(FileOperationError {
                message: format!("Failed to read file because of {} reason", reason),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::file_utils::file_reader::FileReader;

    #[test]
    fn test_reading() {
        let result = FileReader::from_path(
            "/home/sschakraborty/Projects/Gateman/resources/file_utils_test/Sample"
        ).read();
        match result {
            Ok(content) => {
                println!("{}", content);
            }
            Err(reason) => {
                panic!("{}", reason.message);
            }
        }
    }
}
