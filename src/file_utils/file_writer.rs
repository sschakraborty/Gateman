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
        let path = Path::new(self.filepath.as_str());
        File::create(path);
        match File::open(path) {
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
    use crate::file_utils::file_writer::FileWriter;

    #[test]
    fn test_writing() {
        let test_payload = "Hello World! This is a file write test!".to_string();
        let result = FileWriter::from_path(
            "/home/sschakraborty/Projects/Gateman/resources/file_utils_test/SampleWrittenFile"
        ).write(&test_payload);
    }
}
