use std::sync::Mutex;
use tokio::sync::mpsc;

pub type ChannelId = u32;

/// A wrapper around Tokio's mpsc channel
#[derive(Debug)]
pub struct AutoChannel {
    pub id: ChannelId,
    pub tx: mpsc::Sender<i32>,
    pub rx: Mutex<mpsc::Receiver<i32>>,
}

impl AutoChannel {
    pub fn new(id: ChannelId, capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self {
            id,
            tx,
            rx: Mutex::new(rx),
        }
    }
}
