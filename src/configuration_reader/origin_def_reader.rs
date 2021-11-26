use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RateLimiterAlgorithm {
    TokenBucket,
    LeakyBucket,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimeUnit {
    Minute,
    Second,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RateLimiterConfig {
    algorithm: RateLimiterAlgorithm,
    time_unit: TimeUnit,
    req_per_time_unit: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Server {
    hostname: String,
    port: u16,
    secure: bool,
    verify_cert: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OriginSpecification {
    rate_limiter: RateLimiterConfig,
    servers: Vec<Server>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Origin {
    origin_id: String,
    origin_name: String,
    origin_desc: String,
    specification: OriginSpecification,
}

impl Origin {
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
    pub fn has_id(&self, origin_id: &String) -> bool {
        self.origin_id.eq(origin_id)
    }
}

#[cfg(test)]
mod test {
    use std::io::Read;

    use crate::configuration_reader::origin_def_reader::{Origin, RateLimiterAlgorithm, TimeUnit};

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
            RateLimiterAlgorithm::TokenBucket,
            origin.specification.rate_limiter.algorithm
        );
        assert_eq!(
            TimeUnit::Minute,
            origin.specification.rate_limiter.time_unit
        );
        assert_eq!(200, origin.specification.rate_limiter.req_per_time_unit);
        assert_eq!("localhost", origin.specification.servers[0].hostname);
        assert_eq!(80, origin.specification.servers[0].port);
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
            RateLimiterAlgorithm::TokenBucket,
            origin.specification.rate_limiter.algorithm
        );
        assert_eq!(
            TimeUnit::Minute,
            origin.specification.rate_limiter.time_unit
        );
        assert_eq!(200, origin.specification.rate_limiter.req_per_time_unit);
        assert_eq!("localhost", origin.specification.servers[0].hostname);
        assert_eq!(80, origin.specification.servers[0].port);
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
        assert_eq!(String::from("{\n  \"origin_id\": \"RFX829635\",\n  \"origin_name\": \"Sample Origin\",\n  \"origin_desc\": \"Some nice origin description that can be pretty long\",\n  \"specification\": {\n    \"rate_limiter\": {\n      \"algorithm\": \"TokenBucket\",\n      \"time_unit\": \"Minute\",\n      \"req_per_time_unit\": 200\n    },\n    \"servers\": [\n      {\n        \"hostname\": \"localhost\",\n        \"port\": 80,\n        \"secure\": true,\n        \"verify_cert\": false\n      }\n    ]\n  }\n}"), origin.to_json_pretty().unwrap());
    }
}
