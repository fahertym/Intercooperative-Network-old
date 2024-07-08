#[cfg(test)]
mod tests {
    use super::super::consensus::PoCConsensus;

    #[test]
    fn test_reputation_update() {
        let mut consensus = PoCConsensus::new(0.5, 0.66);
        consensus.add_member("Alice".to_string());
        consensus.update_reputation("Alice", 0.5);
        assert_eq!(consensus.get_reputation("Alice"), Some(1.5));
    }

    #[test]
    fn test_reputation_cap() {
        let mut consensus = PoCConsensus::new(0.5, 0.66);
        consensus.add_member("Bob".to_string());
        consensus.update_reputation("Bob", 15.0);
        assert_eq!(consensus.get_reputation("Bob"), Some(10.0)); // Max reputation
    }

    #[test]
    fn test_reputation_floor() {
        let mut consensus = PoCConsensus::new(0.5, 0.66);
        consensus.add_member("Charlie".to_string());
        consensus.update_reputation("Charlie", -2.0);
        assert_eq!(consensus.get_reputation("Charlie"), Some(0.0)); // Min reputation
    }

    #[test]
    fn test_slashing() {
        let mut consensus = PoCConsensus::new(0.5, 0.66);
        consensus.add_member("Dave".to_string());
        consensus.slash_reputation("Dave", "critical_offense");
        assert_eq!(consensus.get_reputation("Dave"), Some(0.0));
    }

    #[test]
    fn test_rehabilitation() {
        let mut consensus = PoCConsensus::new(0.5, 0.66);
        consensus.add_member("Eve".to_string());
        consensus.update_reputation("Eve", -0.6); // Reputation becomes 0.4
        consensus.rehabilitate_members();
        assert!(consensus.get_reputation("Eve").unwrap() > 0.4);
    }
}