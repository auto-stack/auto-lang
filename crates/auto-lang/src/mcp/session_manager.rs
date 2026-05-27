// MCP Session Manager — per-agent VM session lifecycle

use crate::autovm_persistent::AutovmReplSession;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

fn generate_session_id() -> String {
    let count = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    let rand_part: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    format!("ses_{:04x}{:04x}", count & 0xFFFF, rand_part & 0xFFFF)
}

pub struct SessionManager {
    sessions: HashMap<String, VmSession>,
}

struct VmSession {
    session: AutovmReplSession,
    created_at: Instant,
    last_active: Instant,
    sandbox: bool,
    /// Accumulated source code for auto_snapshot and auto_patch
    source_history: Vec<String>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self { sessions: HashMap::new() }
    }

    pub fn create(&mut self, sandbox: bool) -> String {
        let id = generate_session_id();
        let now = Instant::now();
        self.sessions.insert(id.clone(), VmSession {
            session: AutovmReplSession::new(),
            created_at: now,
            last_active: now,
            sandbox,
            source_history: Vec::new(),
        });
        id
    }

    pub fn get(&mut self, id: &str) -> Option<&mut AutovmReplSession> {
        let entry = self.sessions.get_mut(id)?;
        entry.last_active = Instant::now();
        Some(&mut entry.session)
    }

    pub fn reset(&mut self, id: &str) -> bool {
        if let Some(entry) = self.sessions.get_mut(id) {
            entry.session.reset();
            entry.last_active = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn delete(&mut self, id: &str) -> bool {
        self.sessions.remove(id).is_some()
    }

    pub fn exists(&self, id: &str) -> bool {
        self.sessions.contains_key(id)
    }

    pub fn cleanup_expired(&mut self, max_idle: Duration) {
        let now = Instant::now();
        self.sessions.retain(|_, entry| now.duration_since(entry.last_active) < max_idle);
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Record source code that was successfully executed in a session
    pub fn append_source(&mut self, id: &str, code: &str) {
        if let Some(entry) = self.sessions.get_mut(id) {
            entry.source_history.push(code.to_string());
            entry.last_active = Instant::now();
        }
    }

    /// Get the accumulated source code for a session (for snapshot)
    pub fn get_source(&self, id: &str) -> Option<String> {
        self.sessions.get(id).map(|entry| entry.source_history.join("\n\n"))
    }

    /// Rebuild a session from scratch with patched source.
    /// Returns false if session not found.
    pub fn rebuild_with_source(&mut self, id: &str, new_source: &str) -> bool {
        let entry = match self.sessions.get_mut(id) {
            Some(e) => e,
            None => return false,
        };
        entry.session = AutovmReplSession::new();
        entry.source_history = vec![new_source.to_string()];
        entry.last_active = Instant::now();
        true
    }
}
