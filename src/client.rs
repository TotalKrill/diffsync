use twox_hash::XxHash64;

use super::*;

#[cfg_attr(feature = "bevy_support", derive(bevy::prelude::Resource))]
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
        let mut h = XxHash64::with_seed(1337);
        self.state.hash(&mut h);
        h.finish()
    }

    pub fn update_request(&self) -> ClientUpdateRequest<ID> {
        ClientUpdateRequest {
            id: self.id.clone(),
            current_hash: self.calculate_hash(),
        }
    }

    pub fn apply_update(
        &mut self,
        client_update: ClientUpdate<STATE::Repr>,
    ) -> Result<(), UpdateError> {
        let currenthash = self.calculate_hash();
        match client_update {
            ClientUpdate::Complete {
                complete_diff,
                newhash,
            } => {
                self.state = STATE::identity();
                let before_apply = self.calculate_hash();
                log::info!("before apply: {before_apply:X}");
                self.state.apply(&complete_diff);
                let myhash = self.calculate_hash();
                log::info!("calucalted hash: {myhash:X}");
                log::info!("expected hash: {newhash:X}");
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
