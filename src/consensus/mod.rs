#[derive(Default)]
pub struct PoCConsensus {
    pub threshold: f64,
    pub members: Vec<String>,
}

impl PoCConsensus {
    pub fn new(threshold: f64, _vote_threshold: f64) -> Self {
        PoCConsensus {
            threshold,
            members: vec![],
        }
    }

    pub fn add_member(&mut self, member: String, _is_organization: bool) {
        self.members.push(member);
    }
}
