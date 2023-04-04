use diff::Diff;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::DefaultHasher, BTreeMap},
    hash::{Hash, Hasher},
};

// implementation

#[derive(Default)]
pub struct Server<STATE, ID>
where
    STATE: Hash + Diff,
    ID: Ord,
{
    pub state: STATE,
    // Keep states
    client_states: BTreeMap<ID, ClientState<STATE>>,
}

impl<STATE: Hash + Diff, ID: Ord> Server<STATE, ID> {
    /// Create a new server instance using the supplied data
    pub fn new(data: STATE) -> Self {
        Self {
            state: data,
            client_states: Default::default(),
        }
    }

    /// Allows for the server to forget a client. For example, one might keep track of when a client
    /// last requested an update, and remove it if that was too long ago
    pub fn forget_client(&mut self, id: ID) {
        self.client_states.remove(&id);
    }

    /// Get access to the server state data for reading
    pub fn get_state(&self) -> &STATE {
        &self.state
    }

    /// Get access to the server state data for mutation
    pub fn get_state_mut(&mut self) -> &mut STATE {
        &mut self.state
    }
}

impl<STATE: Hash + Clone + Diff, ID: Ord> Server<STATE, ID> {
    fn calculate_hash(&self) -> u64 {
        let mut h = DefaultHasher::new();
        self.state.hash(&mut h);
        h.finish()
    }

    pub fn get_client_diff(&mut self, request: ClientUpdateRequest<ID>) -> ClientUpdate<STATE> {
        let serverhash = self.calculate_hash();

        let upd = match self.client_states.get(&request.id) {
            Some(clientstate) => {
                // we know of this client, we know of a state that was last request, we need to verify that the clients current state is the one we have
                if clientstate.hash == request.current_hash {
                    ClientUpdate::Diff {
                        diff: clientstate.state.diff(&self.state),
                        newhash: serverhash,
                        oldhash: request.current_hash,
                    }
                } else {
                    // send complete new update if the clients percieced hash and what the server thinks the client has differs
                    ClientUpdate::Complete {
                        state: self.state.clone(),
                        newhash: serverhash,
                    }
                }
            }
            None => {
                // new clients gets a complete hash
                ClientUpdate::Complete {
                    state: self.state.clone(),
                    newhash: serverhash,
                }
            }
        };

        // After the update, assume that the client has updated information
        self.client_states.insert(
            request.id,
            ClientState {
                state: self.state.clone(),
                hash: serverhash,
            },
        );
        upd
    }
}

#[derive(Debug, Default)]
pub struct ClientState<STATE> {
    state: STATE,
    hash: u64,
}

#[derive(Deserialize, Serialize)]
pub enum ClientUpdate<STATE: Diff> {
    /// The clients need a complete update, for whatever reason
    Complete { state: STATE, newhash: u64 },
    /// only a diff needs to be applied, and equal hash means that the diff applied succesfully
    Diff {
        diff: STATE::Repr,
        newhash: u64,
        oldhash: u64,
    },
}

#[derive()]
pub struct Client<STATE: Default, ID> {
    id: ID,
    pub state: STATE,
}

impl<STATE: Hash + Diff + Default, ID: Clone> Client<STATE, ID> {
    /// Create a new client with the given id, the ID is used to differentiate on the server side
    pub fn with_id(id: ID) -> Self {
        Self {
            id,
            state: Default::default(),
        }
    }
    pub fn id(&self) -> ID {
        self.id.clone()
    }

    fn calculate_hash(&self) -> u64 {
        let mut h = DefaultHasher::new();
        self.state.hash(&mut h);
        h.finish()
    }

    pub fn update_request(&self) -> ClientUpdateRequest<ID> {
        ClientUpdateRequest {
            id: self.id.clone(),
            current_hash: self.calculate_hash(),
        }
    }

    pub fn apply_update(&mut self, client_update: ClientUpdate<STATE>) -> Result<(), UpdateError> {
        let currenthash = self.calculate_hash();
        match client_update {
            ClientUpdate::Complete { state, newhash } => {
                self.state = state;
                // state
                if newhash == self.calculate_hash() {
                    return Ok(());
                } else {
                    return Err(UpdateError::HashResultDiff);
                }
            }
            ClientUpdate::Diff {
                diff,
                newhash,
                oldhash,
            } => {
                if currenthash == oldhash {
                    self.state.apply(&diff);

                    if newhash == self.calculate_hash() {
                        return Ok(());
                    } else {
                        return Err(UpdateError::HashResultDiff);
                    }
                } else {
                    return Err(UpdateError::InvalidUpdateStartState);
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum UpdateError {
    InvalidUpdateStartState,
    HashResultDiff,
}

#[derive(Serialize, Deserialize, Debug)]
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
        let mut client = Client::with_id(1337);
        // the servers state updates frequently, each time the client requests
        let mut server: Server<Data, u32> = Server::default();
        for i in 0..100 {
            server
                .state
                .anchors
                .insert(i, Anchor::random_variant(&mut rng));
        }

        println!("Client states: {:#?}", server.client_states);

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
                state: _,
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
