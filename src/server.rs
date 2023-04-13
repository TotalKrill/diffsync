use dashmap::DashMap;
use twox_hash::XxHash64;

use super::*;

#[derive(Default)]
pub struct Server<STATE, ID>
where
    STATE: Diff,
    ID: Hash + Ord,
{
    pub state: STATE,
    // Keep states
    client_states: DashMap<ID, ClientState<STATE>>,
}

impl<STATE: Diff, ID: Hash + Ord> Server<STATE, ID> {
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

impl<STATE: Hash + Clone + Diff, ID: Hash + Ord> Server<STATE, ID> {
    fn calculate_hash(&self) -> u64 {
        let mut h = XxHash64::with_seed(1337);
        self.state.hash(&mut h);
        h.finish()
    }

    pub fn get_client_diff(&self, request: ClientUpdateRequest<ID>) -> ClientUpdate<STATE::Repr> {
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
                    let new: STATE = STATE::identity();
                    let complete_diff = new.diff(&self.state);
                    // send complete new update if the clients percieced hash and what the server thinks the client has differs
                    ClientUpdate::Complete {
                        complete_diff,
                        newhash: serverhash,
                    }
                }
            }
            None => {
                let new: STATE = STATE::identity();
                let complete_diff = new.diff(&self.state);
                // send complete new update if the clients percieced hash and what the server thinks the client has differs
                ClientUpdate::Complete {
                    complete_diff,
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
