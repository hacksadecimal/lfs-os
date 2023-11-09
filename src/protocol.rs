use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProtocolError {
    code: i32,
    message: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Operation {
    #[serde(rename = "upload")]
    Upload,
    #[serde(rename = "download")]
    Download,
}

impl ProtocolError {
    pub fn new(code: i32, message: String) -> Self {
        Self { code, message }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "event")]
pub enum Request {
    #[serde(rename = "terminate")]
    Terminate,
    #[serde(rename = "download")]
    Download { oid: String, size: usize },
    #[serde(rename = "upload")]
    Upload {
        oid: String,
        size: usize,
        path: String,
    },
    #[serde(rename = "init")]
    Init {
        operation: Operation,
        remote: String,
        concurrent: bool,
        concurrenttransfers: i32,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InitResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ProtocolError>,
}

impl InitResponse {
    pub fn new(error: Option<ProtocolError>) -> Self {
        Self { error }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum TransferResponse {
    Successful {
        event: String,
        oid: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    Error {
        event: String,
        oid: String,
        error: ProtocolError,
    },
}

impl TransferResponse {
    pub fn new(oid: String, response: Result<Option<String>, ProtocolError>) -> Self {
        let event = String::from("complete");
        match response {
            Ok(path) => Self::Successful { event, oid, path },
            Err(error) => Self::Error { event, oid, error },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProgressResponse {
    event: String,
    oid: String,
    #[serde(rename = "bytesSoFar")]
    bytes_so_far: usize,
    #[serde(rename = "bytesSinceLast")]
    bytes_since_last: usize,
}

impl ProgressResponse {
    pub fn new(oid: String, bytes_so_far: usize, bytes_since_last: usize) -> Self {
        Self {
            event: String::from("progress"),
            oid,
            bytes_so_far,
            bytes_since_last,
        }
    }
}

pub trait Response: Serialize {
    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl Response for InitResponse {}
impl Response for TransferResponse {}
impl Response for ProgressResponse {}
