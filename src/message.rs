use serde_derive::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub message: String,
}

impl Request {
    pub fn new<T>(id: T, message: T) -> Request
    where
        T: ToString,
    {
        Request {
            id: id.to_string(),
            message: message.to_string(),
        }
    }

    pub fn to_serialized_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }

    pub fn serialize_to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommandStatus {
    Ok,
    Processing,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub status: CommandStatus,
    pub message: String,
}

impl Response {
    pub fn new<T>(id: T, status: CommandStatus, message: T) -> Response
    where
        T: ToString,
    {
        Response {
            id: id.to_string(),
            status,
            message: message.to_string(),
        }
    }

    pub fn to_serialized_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }

    pub fn serialize_to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serde_to_and_from_string_works() {
        let request = Request::new("some-request-id", "some content message");
        let stringified_request = request.to_serialized_string().unwrap();

        assert_eq!(
            serde_json::from_str::<Request>(&stringified_request).unwrap(),
            request
        );
    }

    #[test]
    fn serde_from_bytes_works() {
        let request = Request::new("some-request-id", "some content message");
        let stringified_request = request.to_serialized_string().unwrap();
        let request_as_bytes = stringified_request.as_bytes();
        assert_eq!(
            serde_json::from_slice::<Request>(request_as_bytes).unwrap(),
            request
        );
    }

    #[test]
    fn serde_to_bytes_works() {
        let request = Request::new("some-request-id", "some content message");
        let stringified_request = request.to_serialized_string().unwrap();
        let request_as_bytes = stringified_request.as_bytes();
        assert_eq!(serde_json::to_vec(&request).unwrap(), request_as_bytes);
    }

    #[test]
    fn deserialize_error_works() {
        let bad_request = "{\"id\":345,\"message\":\"HeyPatric\"}";
        assert!(serde_json::from_str::<Request>(&bad_request).is_err())
    }
}
