use log::{debug, trace};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimeUnit {
    Hour,
    Minute,
    Second,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RateLimiterConfig {
    pub(crate) time_unit: TimeUnit,
    pub(crate) req_per_time_unit: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Server {
    pub(crate) hostname: String,
    pub(crate) port: u16,
    pub(crate) secure: bool,
    pub(crate) verify_cert: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OriginSpecification {
    pub(crate) rate_limiter: RateLimiterConfig,
    pub(crate) servers: Vec<Server>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Origin {
    pub(crate) origin_id: String,
    pub(crate) origin_name: String,
    pub(crate) origin_desc: String,
    pub(crate) specification: OriginSpecification,
}

impl Origin {
    pub fn from_json_string(json_payload: &String) -> Result<Self, serde_json::Error> {
        debug!("Constructing Origin from JSON payload!");
        trace!("Trying to create Origin from {}", json_payload);
        serde_json::from_str::<Self>(json_payload.as_str())
    }
    pub fn from_json_str_slice(json_payload: &str) -> Result<Self, serde_json::Error> {
        debug!("Constructing Origin from JSON payload!");
        trace!("Trying to create Origin from {}", json_payload);
        serde_json::from_str::<Self>(json_payload)
    }
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        debug!("Serializing Origin to JSON!");
        serde_json::to_string(self)
    }
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        debug!("Serializing Origin to pretty JSON!");
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod test {
    use std::io::Read;

    use crate::configuration_reader::origin_def_reader::{Origin, TimeUnit};

    #[test]
    fn test_deserialize() {
        let mut file_contents = String::new();
        let path = std::path::Path::new(
            "/home/sschakraborty/Projects/Gateman/resources/definitions/origin_def/origin.json",
        );
        match std::fs::File::open(&path) {
            Ok(mut file) => {
                file.read_to_string(&mut file_contents);
            }
            Err(reason) => {
                panic!("Failed for {}", reason)
            }
        }
        let origin = Origin::from_json_string(&file_contents).unwrap();
        assert_eq!("RFX829635", origin.origin_id);
        assert_eq!("Sample Origin", origin.origin_name);
        assert_eq!(
            "Some nice origin description that can be pretty long",
            origin.origin_desc
        );
        assert_eq!(
            TimeUnit::Minute,
            origin.specification.rate_limiter.time_unit
        );
        assert_eq!(200, origin.specification.rate_limiter.req_per_time_unit);
        assert_eq!("localhost", origin.specification.servers[0].hostname);
        assert_eq!(8000, origin.specification.servers[0].port);
        assert_eq!(true, origin.specification.servers[0].secure);
        assert_eq!(false, origin.specification.servers[0].verify_cert);

        let json_payload = origin.to_json().unwrap();
        let origin = Origin::from_json_str_slice(json_payload.as_str()).unwrap();
        assert_eq!("RFX829635", origin.origin_id);
        assert_eq!("Sample Origin", origin.origin_name);
        assert_eq!(
            "Some nice origin description that can be pretty long",
            origin.origin_desc
        );
        assert_eq!(
            TimeUnit::Minute,
            origin.specification.rate_limiter.time_unit
        );
        assert_eq!(200, origin.specification.rate_limiter.req_per_time_unit);
        assert_eq!("localhost", origin.specification.servers[0].hostname);
        assert_eq!(8000, origin.specification.servers[0].port);
        assert_eq!(true, origin.specification.servers[0].secure);
        assert_eq!(false, origin.specification.servers[0].verify_cert);
    }

    #[test]
    fn test_serialize() {
        let mut file_contents = String::new();
        let path = std::path::Path::new(
            "/home/sschakraborty/Projects/Gateman/resources/definitions/origin_def/origin.json",
        );
        match std::fs::File::open(&path) {
            Ok(mut file) => {
                file.read_to_string(&mut file_contents);
            }
            Err(reason) => {
                panic!("Failed for {}", reason)
            }
        }
        let origin = Origin::from_json_string(&file_contents).unwrap();
        assert_eq!(String::from("{\n  \"origin_id\": \"RFX829635\",\n  \"origin_name\": \"Sample Origin\",\n  \"origin_desc\": \"Some nice origin description that can be pretty long\",\n  \"specification\": {\n    \"rate_limiter\": {\n      \"time_unit\": \"Minute\",\n      \"req_per_time_unit\": 200\n    },\n    \"servers\": [\n      {\n        \"hostname\": \"localhost\",\n        \"port\": 8000,\n        \"secure\": true,\n        \"verify_cert\": false\n      }\n    ]\n  }\n}"), origin.to_json_pretty().unwrap());
    }
}
