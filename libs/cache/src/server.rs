use std::path::PathBuf;

use crate::rpc::cache::*;
use tonic::Response;

pub struct CacheServer {
    root: PathBuf,
}

pub struct RemoteCache {}

#[tonic::async_trait]
impl remote_cache_server::RemoteCache for RemoteCache {
    async fn get(
        &self,
        request: tonic::Request<CacheGetRequest>,
    ) -> Result<tonic::Response<CacheGetReply>, tonic::Status> {
        Ok(Response::new(CacheGetReply {
            entry_status: Some(cache_get_reply::EntryStatus::Unassigned(())),
        }))
    }
}
