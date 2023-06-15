use reqwest::IntoUrl;

use crate::rest_api::{CacheGetRequest, CacheGetResponse, CachePutRequest, CachePutResponse};

pub fn remote_cache_get(url: impl IntoUrl, id: impl Into<String>, key: Vec<u8>) -> Option<Vec<u8>> {
    let req = flexbuffers::to_vec(CacheGetRequest {
        path: "test".to_string(),
        id: id.into(),
        key,
    })
    .unwrap();

    let mut url = url.into_url().unwrap();
    url.set_path("get");

    let client = reqwest::blocking::Client::new();
    let body = client.post(url).body(req).send().unwrap().bytes().unwrap();
    let res: CacheGetResponse = flexbuffers::from_slice(&body).unwrap();
    res.value
}

pub fn remote_cache_put(url: impl IntoUrl, id: impl Into<String>, key: Vec<u8>, value: Vec<u8>) {
    let req = flexbuffers::to_vec(CachePutRequest {
        path: "test".to_string(),
        id: id.into(),
        key: key.clone(),
        value,
    })
    .unwrap();

    let mut url = url.into_url().unwrap();
    url.set_path("put");

    let client = reqwest::blocking::Client::new();
    let body = client.post(url).body(req).send().unwrap().bytes().unwrap();
    let res: CachePutResponse = flexbuffers::from_slice(&body).unwrap();

    assert_eq!(res.key, key);
}
