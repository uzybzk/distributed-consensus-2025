// Raft consensus algorithm implementation
// Based on the Raft paper by Diego Ongaro and John Ousterhout

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub command: String,
    pub id: Uuid,
}

#[derive(Debug, Clone)]
pub struct RaftState {
    // Persistent state on all servers
    pub current_term: u64,
    pub voted_for: Option<String>,
    pub log: Vec<LogEntry>,
    
    // Volatile state on all servers
    pub commit_index: u64,
    pub last_applied: u64,
    
    // Volatile state on leaders
    pub next_index: HashMap<String, u64>,
    pub match_index: HashMap<String, u64>,
}

impl Default for RaftState {
    fn default() -> Self {
        Self {
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRequest {
    pub term: u64,
    pub candidate_id: String,
    pub last_log_index: u64,
    pub last_log_term: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResponse {
    pub term: u64,
    pub vote_granted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesRequest {
    pub term: u64,
    pub leader_id: String,
    pub prev_log_index: u64,
    pub prev_log_term: u64,
    pub entries: Vec<LogEntry>,
    pub leader_commit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesResponse {
    pub term: u64,
    pub success: bool,
}

impl RaftState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn last_log_index(&self) -> u64 {
        self.log.len() as u64
    }
    
    pub fn last_log_term(&self) -> u64 {
        if let Some(entry) = self.log.last() {
            entry.term
        } else {
            0
        }
    }
    
    pub fn handle_vote_request(&mut self, req: VoteRequest, node_id: &str) -> VoteResponse {
        let mut vote_granted = false;
        
        // Update current term if necessary
        if req.term > self.current_term {
            self.current_term = req.term;
            self.voted_for = None;
        }
        
        // Grant vote if:
        // 1. Haven't voted for anyone else in this term
        // 2. Candidate's log is at least as up-to-date as ours
        if req.term >= self.current_term 
            && (self.voted_for.is_none() || self.voted_for.as_ref() == Some(&req.candidate_id))
            && self.is_log_up_to_date(req.last_log_index, req.last_log_term) {
            
            vote_granted = true;
            self.voted_for = Some(req.candidate_id);
        }
        
        VoteResponse {
            term: self.current_term,
            vote_granted,
        }
    }
    
    pub fn handle_append_entries(&mut self, req: AppendEntriesRequest) -> AppendEntriesResponse {
        let mut success = false;
        
        // Update current term if necessary
        if req.term > self.current_term {
            self.current_term = req.term;
            self.voted_for = None;
        }
        
        // Reply false if term < currentTerm
        if req.term >= self.current_term {
            // Reply false if log doesn't contain an entry at prevLogIndex
            // whose term matches prevLogTerm
            if req.prev_log_index == 0 || 
               (req.prev_log_index <= self.log.len() as u64 && 
                self.log.get((req.prev_log_index - 1) as usize)
                    .map_or(0, |entry| entry.term) == req.prev_log_term) {
                
                success = true;
                
                // If an existing entry conflicts with a new one, delete the existing
                // entry and all that follow it
                if req.prev_log_index < self.log.len() as u64 {
                    self.log.truncate(req.prev_log_index as usize);
                }
                
                // Append any new entries not already in the log
                self.log.extend(req.entries);
                
                // Update commit index
                if req.leader_commit > self.commit_index {
                    self.commit_index = std::cmp::min(req.leader_commit, self.log.len() as u64);
                }
            }
        }
        
        AppendEntriesResponse {
            term: self.current_term,
            success,
        }
    }
    
    fn is_log_up_to_date(&self, last_log_index: u64, last_log_term: u64) -> bool {
        let our_last_term = self.last_log_term();
        let our_last_index = self.last_log_index();
        
        // Candidate's log is more up-to-date if:
        // 1. Last log term is higher, OR
        // 2. Last log terms are equal but candidate's log is longer
        last_log_term > our_last_term || 
        (last_log_term == our_last_term && last_log_index >= our_last_index)
    }
    
    pub fn append_entry(&mut self, command: String, term: u64) -> LogEntry {
        let entry = LogEntry {
            term,
            index: self.log.len() as u64 + 1,
            command,
            id: Uuid::new_v4(),
        };
        
        self.log.push(entry.clone());
        entry
    }
}