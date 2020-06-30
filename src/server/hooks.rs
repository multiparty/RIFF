use crate::server::datastructure::intervals::*;
pub struct serverHooks {

}

impl serverHooks {
    pub fn trackFreeIds (party_count: u64) -> intervals {
        return intervals_fn(1, party_count)
    }
}