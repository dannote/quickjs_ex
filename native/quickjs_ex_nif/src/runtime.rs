use crate::worker;
use rustler::{Encoder, LocalPid, OwnedEnv};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};

pub struct CallbackRegistry {
    pending: Mutex<HashMap<u64, mpsc::Sender<String>>>,
    next_id: AtomicU64,
}

impl CallbackRegistry {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn register(&self) -> (u64, mpsc::Receiver<String>) {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel();
        self.pending.lock().unwrap().insert(id, tx);
        (id, rx)
    }

    pub fn respond(&self, id: u64, result: String) -> bool {
        if let Some(tx) = self.pending.lock().unwrap().remove(&id) {
            tx.send(result).is_ok()
        } else {
            false
        }
    }

    pub fn clear(&self) {
        self.pending.lock().unwrap().clear();
    }
}

pub struct Runtime {
    sender: std::sync::mpsc::Sender<worker::Message>,
    pub callbacks: Arc<CallbackRegistry>,
}

impl Runtime {
    pub fn new(sender: std::sync::mpsc::Sender<worker::Message>) -> Self {
        Self {
            sender,
            callbacks: Arc::new(CallbackRegistry::new()),
        }
    }

    pub fn send(
        &self,
        msg: worker::Message,
    ) -> Result<(), std::sync::mpsc::SendError<worker::Message>> {
        self.sender.send(msg)
    }
}

#[rustler::resource_impl]
impl rustler::Resource for Runtime {}

pub fn send_to_pid<T>(pid: &LocalPid, data: T)
where
    T: Encoder,
{
    let _ = OwnedEnv::new().send_and_clear(pid, |_env| data);
}
