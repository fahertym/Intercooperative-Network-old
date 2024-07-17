use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct PoCConsensus {
    pub members: Vec<Member>,
    pub threshold: f64,
}

impl PoCConsensus {
    pub fn new(threshold: f64, _quorum: f64) -> Self {
        PoCConsensus {
            members: Vec::new(),
            threshold,
        }
    }

    pub fn add_member(&mut self, member_id: String, is_validator: bool) {
        self.members.push(Member { id: member_id, is_validator });
    }
}

#[derive(Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub is_validator: bool,
}
