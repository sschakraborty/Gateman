use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::file_utils::file_error::FileOperationError;

pub(crate) struct FileWriter {
    filepath: String,
}

impl FileWriter {
    pub(crate) fn from_path(path: &str) -> Self {
        FileWriter {
            filepath: String::from(path),
        }
    }
    pub(crate) fn write(&self, content: &String) -> Result<(), FileOperationError> {
        match File::create(Path::new(self.filepath.as_str())) {
            Ok(mut file) => {
                file.write_all(content.as_bytes());
                Result::Ok(())
            }
            Err(reason) => Result::Err(FileOperationError {
                message: format!("Failed to write to file because of {} reason", reason),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::file_utils::file_reader::FileReader;
    use crate::file_utils::file_writer::FileWriter;

    #[test]
    fn test_writing() {
        let test_payload = "Hello World! This is a file write test!".to_string();
        let filepath =
            "/home/sschakraborty/Projects/Gateman/resources/file_utils_test/SampleWrittenFile";
        let result = FileWriter::from_path(filepath).write(&test_payload);
        let read_result = FileReader::from_path(filepath).read();
        match read_result {
            Ok(content) => {
                assert_eq!(test_payload, content);
            }
            Err(_) => {
                panic!();
            }
        }
    }
}
