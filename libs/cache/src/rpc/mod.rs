//! Messages and gRPC services for the cache servers.

use std::marker::PhantomData;

use bytes::{Buf, BufMut};
use serde::{Serialize, de::DeserializeOwned};
use tonic::Status;
use tonic::codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};

/// The reply of RPCs that return no data.
pub type Empty = ();

pub mod local {
    //! The local cache API, which stores values on a filesystem shared with the client.

    use serde::{Deserialize, Serialize};

    /// Requests the status of an entry, optionally assigning the client to generate it.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct GetRequest {
        pub namespace: String,
        pub key: Vec<u8>,
        pub assign: bool,
    }

    /// The status of a requested entry.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub enum GetReply {
        Unassigned,
        Assign(AssignReply),
        Loading,
        Ready(ReadyReply),
    }

    /// An assignment to generate an entry and write it to `path`.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct AssignReply {
        pub id: u64,
        pub path: String,
        pub heartbeat_interval_ms: u64,
    }

    /// A handle to a ready entry stored at `path`.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ReadyReply {
        pub id: u64,
        pub path: String,
    }

    /// Releases the handle with the given ID.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct DropRequest {
        pub id: u64,
    }

    /// Keeps the assignment with the given ID alive.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct HeartbeatRequest {
        pub id: u64,
    }

    /// Marks the assignment with the given ID as written.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct DoneRequest {
        pub id: u64,
    }

    include!(concat!(env!("OUT_DIR"), "/local.LocalCache.rs"));
}

pub mod remote {
    //! The remote cache API, which sends values over the network.

    use serde::{Deserialize, Serialize};

    /// Requests the status of an entry, optionally assigning the client to generate it.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct GetRequest {
        pub namespace: String,
        pub key: Vec<u8>,
        pub assign: bool,
    }

    /// The status of a requested entry.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub enum GetReply {
        Unassigned,
        Assign(AssignReply),
        Loading,
        /// The serialized value of the entry.
        Ready(Vec<u8>),
    }

    /// An assignment to generate an entry.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct AssignReply {
        pub id: u64,
        pub heartbeat_interval_ms: u64,
    }

    /// Keeps the assignment with the given ID alive.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct HeartbeatRequest {
        pub id: u64,
    }

    /// Stores the serialized value for the assignment with the given ID.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct SetRequest {
        pub id: u64,
        pub value: Vec<u8>,
    }

    include!(concat!(env!("OUT_DIR"), "/remote.RemoteCache.rs"));
}

/// A gRPC codec that encodes messages with [`postcard`].
#[derive(Debug)]
pub struct PostcardCodec<T, U>(PhantomData<(T, U)>);

impl<T, U> Default for PostcardCodec<T, U> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T, U> Codec for PostcardCodec<T, U>
where
    T: Serialize + Send + 'static,
    U: DeserializeOwned + Send + 'static,
{
    type Encode = T;
    type Decode = U;
    type Encoder = PostcardEncoder<T>;
    type Decoder = PostcardDecoder<U>;

    fn encoder(&mut self) -> Self::Encoder {
        PostcardEncoder(PhantomData)
    }

    fn decoder(&mut self) -> Self::Decoder {
        PostcardDecoder(PhantomData)
    }
}

/// The encoder of a [`PostcardCodec`].
#[derive(Debug)]
pub struct PostcardEncoder<T>(PhantomData<T>);

impl<T: Serialize> Encoder for PostcardEncoder<T> {
    type Item = T;
    type Error = Status;

    fn encode(&mut self, item: T, buf: &mut EncodeBuf<'_>) -> Result<(), Status> {
        postcard::to_io(&item, buf.writer())
            .map(|_| ())
            .map_err(|e| Status::internal(format!("failed to encode message: {e}")))
    }
}

/// The decoder of a [`PostcardCodec`].
#[derive(Debug)]
pub struct PostcardDecoder<U>(PhantomData<U>);

impl<U: DeserializeOwned> Decoder for PostcardDecoder<U> {
    type Item = U;
    type Error = Status;

    fn decode(&mut self, buf: &mut DecodeBuf<'_>) -> Result<Option<U>, Status> {
        // Each call receives exactly one message, which is empty for `()`.
        let bytes = buf.copy_to_bytes(buf.remaining());
        postcard::from_bytes(&bytes)
            .map(Some)
            .map_err(|e| Status::invalid_argument(format!("failed to decode message: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::{local, remote};

    #[test]
    fn messages_round_trip() {
        let reply = remote::GetReply::Ready(vec![0, 1, 255]);
        let bytes = postcard::to_stdvec(&reply).unwrap();
        assert_eq!(
            postcard::from_bytes::<remote::GetReply>(&bytes).unwrap(),
            reply
        );

        let reply = local::GetReply::Assign(local::AssignReply {
            id: 7,
            path: "/tmp/x".into(),
            heartbeat_interval_ms: 2000,
        });
        let bytes = postcard::to_stdvec(&reply).unwrap();
        assert_eq!(
            postcard::from_bytes::<local::GetReply>(&bytes).unwrap(),
            reply
        );

        let bytes = postcard::to_stdvec(&()).unwrap();
        postcard::from_bytes::<()>(&bytes).unwrap();
    }
}
