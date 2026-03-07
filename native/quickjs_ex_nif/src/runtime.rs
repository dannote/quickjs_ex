use crate::worker;
use rustler::{Encoder, LocalPid, OwnedEnv};

pub struct Runtime {
    sender: std::sync::mpsc::Sender<worker::Message>,
}

impl Runtime {
    pub fn new(sender: std::sync::mpsc::Sender<worker::Message>) -> Self {
        Self { sender }
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
