use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheGetRequest {
    pub path: String,
    pub id: String,
    pub key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheGetResponse {
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePutRequest {
    pub path: String,
    pub id: String,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePutResponse {
    pub key: Vec<u8>,
}
