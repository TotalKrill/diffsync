#![feature(hasher_prefixfree_extras)]

use dashmap::DashMap;
pub use diff::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    hash::{Hash, Hasher},
};

pub use structs::SimpleDiff;

// implementation
pub mod client;
pub mod customhash;
pub mod server;
pub mod structs;

pub use concmap::ConcMap;

/// Concurrent wrapper for dashmap, to implement all ze traits on
pub mod concmap;

#[cfg_attr(feature = "impl_schemars", derive(schemars::JsonSchema))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum ClientUpdate<T> {
    Complete {
        // Technicalities makes this more feasible than return the entire STATE, especially for those cases when STATE is in an Arc
        complete_diff: T,
        newhash: u64,
    },
    Diff {
        /// only a diff needs to be applied, and equal hash means that the diff applied succesfully
        diff: T,
        newhash: u64,
        oldhash: u64,
    },
}

#[derive(Debug)]
pub enum UpdateError {
    InvalidUpdateStartState,
    HashResultDiff,
}

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClientUpdateRequest<ID> {
    id: ID,
    current_hash: u64,
}

#[cfg(test)]
mod test {
    use super::*;
    use diff::BTreeMapDiff;
    use pretty_assertions::assert_eq;
    use rand::rngs::ThreadRng;
    use random_variant::RandomVariant;

    pub trait Empty {
        fn empty(&self) -> bool;
    }

    impl<K: Ord + Eq, V: Diff> Empty for BTreeMapDiff<K, V> {
        fn empty(&self) -> bool {
            self.altered.is_empty() && self.removed.is_empty()
        }
    }

    #[derive(
        Deserialize, Serialize, Clone, Hash, Diff, Debug, PartialEq, RandomVariant, Default,
    )]
    #[diff(attr(#[derive(Serialize, Deserialize)]))]
    pub struct Position {
        x: u32,
        y: u32,
    }

    #[derive(
        Deserialize, Serialize, Clone, Hash, Diff, Debug, PartialEq, RandomVariant, Default,
    )]
    #[diff(attr(#[derive(Serialize, Deserialize)]))]
    pub struct Tag {
        position: Position,
        timestamp: usize,
        battery: u32,
    }

    #[derive(
        Deserialize, Serialize, Clone, Hash, Diff, Debug, PartialEq, RandomVariant, Default,
    )]
    #[diff(attr(#[derive(Serialize, Deserialize)]))]
    pub struct Anchor {
        position: Position,
        sid: u32,
    }

    #[derive(Deserialize, Serialize, Diff, Debug, Clone, Hash, PartialEq, Default)]
    #[diff(attr(#[derive(Serialize, Deserialize)]))]
    pub struct Data {
        // #[diff(attr(#[serde(skip_serializing_if = "BTreeMapDiff::empty")]))]
        pub anchors: BTreeMap<u32, Anchor>,
        // #[diff(attr(#[serde(skip_serializing_if = "BTreeMapDiff::empty")]))]
        pub tags: BTreeMap<u32, Tag>,
    }

    #[test]
    fn update_works() {
        let mut rng = ThreadRng::default();

        // clients request updates from the server, after each request, the state should be equal, and hopefulle it does not mean a full state update
        let mut client: client::Client<Data, u32> = client::Client::with_id(1337);
        // the servers state updates frequently, each time the client requests
        let mut server: server::Server<Data, u32> = server::Server::default();
        for i in 0..100 {
            server
                .state
                .anchors
                .insert(i, Anchor::random_variant(&mut rng));
        }

        let request = client.update_request();
        println!("Request: {request:?}");

        let client_update = server.get_client_diff(request);
        let supd = serde_json::to_string(&client_update).unwrap().len();
        println!("Update len: {supd}");
        let bupd = bincode::serialize(&client_update).unwrap().len();
        println!("Binary: {bupd}");

        assert!(client.apply_update(client_update).is_ok());
        assert_eq!(client.state, server.state);

        for i in 0..50 {
            server
                .state
                .anchors
                .insert(i, Anchor::random_variant(&mut rng));
        }

        let request = client.update_request();
        let client_update = server.get_client_diff(request);
        let supd = serde_json::to_string(&client_update).unwrap();
        println!("Update len: {supd}");
        let bupd = bincode::serialize(&client_update).unwrap();
        println!("Binary: {}", bupd.len());
        let apply = bincode::deserialize(&bupd).unwrap();
        // let apply = serde_json::from_str(&supd).unwrap();

        match &client_update {
            ClientUpdate::Complete {
                complete_diff: _,
                newhash: _,
            } => assert!(false, "Should not be a complete update!"),
            ClientUpdate::Diff {
                diff: _,
                newhash: _,
                oldhash: _,
            } => {
                //println!("newhash: {newhash}");
            }
        }

        let res = client.apply_update(apply);
        println!("{res:?}");
        assert_eq!(client.state, server.state);
    }
}
