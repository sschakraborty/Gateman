use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct APISpecification {
    pub(crate) methods: Vec<String>,
    pub(crate) paths: Vec<String>,
    pub(crate) hostnames: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct APIDefinition {
    pub(crate) api_id: String,
    pub(crate) api_name: String,
    pub(crate) api_version: String,
    pub(crate) api_desc: String,
    pub(crate) specification: APISpecification,
    pub(crate) backend_response_timeout: u64,
    pub(crate) origin_id: String,
}

impl APIDefinition {
    pub fn from_json_string(json_payload: &String) -> Result<Self, serde_json::Error> {
        serde_json::from_str::<Self>(json_payload.as_str())
    }
    pub fn from_json_str_slice(json_payload: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str::<Self>(json_payload)
    }
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    pub fn origin_id(&self) -> String {
        self.origin_id.clone()
    }
}

#[cfg(test)]
mod test {
    use std::io::Read;
    use std::path::Path;

    use crate::configuration_reader::api_def_reader::APIDefinition;

    #[test]
    fn test_everything() {
        let mut file_contents = String::new();
        let path = Path::new(
            "/home/sschakraborty/Projects/Gateman/resources/definitions/api_def/sample_api_def.json"
        );
        match std::fs::File::open(&path) {
            Ok(mut file) => {
                file.read_to_string(&mut file_contents);
            }
            Err(reason) => {
                panic!("Failed for {}", reason);
            }
        }

        let api_definition = APIDefinition::from_json_string(&file_contents).unwrap();
        assert_eq!(String::from("some_nice_id"), api_definition.api_id);
        assert_eq!(String::from("Sample API"), api_definition.api_name);
        assert_eq!(String::from("0.1.0"), api_definition.api_version);
        assert_eq!(
            String::from("Some nice description of this beautiful API"),
            api_definition.api_desc
        );
        assert_eq!(2500, api_definition.backend_response_timeout);

        let json_payload_serialized = api_definition.to_json().unwrap();
        let second_api_definition =
            APIDefinition::from_json_str_slice(json_payload_serialized.as_str()).unwrap();
        assert_eq!(String::from("some_nice_id"), second_api_definition.api_id);
        assert_eq!(String::from("Sample API"), second_api_definition.api_name);
        assert_eq!(String::from("0.1.0"), second_api_definition.api_version);
        assert_eq!(
            String::from("Some nice description of this beautiful API"),
            second_api_definition.api_desc
        );
        assert_eq!(2500, second_api_definition.backend_response_timeout);

        let json_pretty_payload_serialized = api_definition.to_json_pretty().unwrap();
        let second_api_definition =
            APIDefinition::from_json_str_slice(json_pretty_payload_serialized.as_str()).unwrap();
        assert_eq!(String::from("some_nice_id"), second_api_definition.api_id);
        assert_eq!(String::from("Sample API"), second_api_definition.api_name);
        assert_eq!(String::from("0.1.0"), second_api_definition.api_version);
        assert_eq!(
            String::from("Some nice description of this beautiful API"),
            second_api_definition.api_desc
        );
        assert_eq!(2500, second_api_definition.backend_response_timeout);

        assert_eq!(
            String::from("GET"),
            second_api_definition.specification.methods[0]
        );
        assert_eq!(
            String::from("POST"),
            second_api_definition.specification.methods[1]
        );
        assert_eq!(
            String::from("PUT"),
            second_api_definition.specification.methods[2]
        );
        assert_eq!(
            String::from("DELETE"),
            second_api_definition.specification.methods[3]
        );
        assert_eq!(
            String::from("OPTION"),
            second_api_definition.specification.methods[4]
        );

        assert_eq!(
            String::from("/some/**/path/*/with/meaning"),
            second_api_definition.specification.paths[0]
        );
        assert_eq!(
            String::from("/some/other/**/path/*/with/**/meaning"),
            second_api_definition.specification.paths[1]
        );

        assert_eq!(
            String::from("*.example.com"),
            second_api_definition.specification.hostnames[0]
        );
    }
}
