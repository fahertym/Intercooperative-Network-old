use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct FibEntry {
    pub name: String,
    pub next_hops: Vec<SocketAddr>,
}

impl FibEntry {
    pub fn new(name: String, next_hop: SocketAddr) -> Self {
        FibEntry {
            name,
            next_hops: vec![next_hop],
        }
    }

    pub fn add_next_hop(&mut self, next_hop: SocketAddr) {
        if !self.next_hops.contains(&next_hop) {
            self.next_hops.push(next_hop);
        }
    }

    pub fn remove_next_hop(&mut self, next_hop: &SocketAddr) {
        self.next_hops.retain(|&x| x != *next_hop);
    }
}

pub struct ForwardingInformationBase {
    entries: HashMap<String, FibEntry>,
}

impl ForwardingInformationBase {
    pub fn new() -> Self {
        ForwardingInformationBase {
            entries: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, name: String, next_hop: SocketAddr) {
        self.entries
            .entry(name.clone())
            .and_modify(|e| e.add_next_hop(next_hop))
            .or_insert_with(|| FibEntry::new(name, next_hop));
    }

    pub fn remove_entry(&mut self, name: &str) {
        self.entries.remove(name);
    }

    pub fn get_next_hops(&self, name: &str) -> Option<&Vec<SocketAddr>> {
        self.entries.get(name).map(|entry| &entry.next_hops)
    }

    pub fn longest_prefix_match(&self, name: &str) -> Option<&FibEntry> {
        let mut longest_match: Option<&FibEntry> = None;
        let mut longest_prefix_len = 0;

        for (prefix, entry) in &self.entries {
            if name.starts_with(prefix) && prefix.len() > longest_prefix_len {
                longest_match = Some(entry);
                longest_prefix_len = prefix.len();
            }
        }

        longest_match
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fib() {
        let mut fib = ForwardingInformationBase::new();
        let addr1: SocketAddr = "127.0.0.1:8000".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:8001".parse().unwrap();

        fib.add_entry("/test".to_string(), addr1);
        fib.add_entry("/test/nested".to_string(), addr2);

        assert_eq!(fib.get_next_hops("/test").unwrap().len(), 1);
        assert_eq!(fib.get_next_hops("/test/nested").unwrap().len(), 1);

        let longest_match = fib.longest_prefix_match("/test/nested/deep");
        assert!(longest_match.is_some());
        assert_eq!(longest_match.unwrap().name, "/test/nested");
    }
}