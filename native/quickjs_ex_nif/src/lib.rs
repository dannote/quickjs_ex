mod atoms;
mod runtime;
mod worker;

use rustler::{Env, LocalPid, NifResult, ResourceArc, Term};
use std::sync::Arc;

#[rustler::nif]
fn start_runtime(env: Env, _pid: LocalPid, browser_stubs: bool) -> rustler::Atom {
    let task_pid = env.pid();
    let (sender, receiver) = std::sync::mpsc::channel::<worker::Message>();

    let opts = worker::WorkerOpts { browser_stubs };

    std::thread::spawn(move || match worker::Worker::new(opts) {
        Ok(mut w) => {
            crate::runtime::send_to_pid(
                &task_pid,
                (atoms::ok(), ResourceArc::new(runtime::Runtime::new(sender))),
            );
            w.run(receiver);
        }
        Err(msg) => {
            crate::runtime::send_to_pid(&task_pid, (atoms::error(), msg));
        }
    });

    atoms::ok()
}

#[rustler::nif(schedule = "DirtyIo")]
fn eval_sync(
    resource: ResourceArc<runtime::Runtime>,
    code: String,
) -> NifResult<(rustler::Atom, String)> {
    let (tx, rx) = std::sync::mpsc::channel();
    resource
        .send(worker::Message::Eval(code, tx))
        .map_err(|_| rustler::Error::Term(Box::new(atoms::dead_runtime())))?;

    match rx.recv() {
        Ok(Ok(val)) => Ok((atoms::ok(), val)),
        Ok(Err(err)) => Ok((atoms::error(), err)),
        Err(_) => Err(rustler::Error::Term(Box::new(atoms::dead_runtime()))),
    }
}

#[rustler::nif]
fn eval_async(
    env: Env,
    from: Term,
    resource: ResourceArc<runtime::Runtime>,
    code: String,
) -> rustler::Atom {
    let pid = env.pid();
    let mut from_env = rustler::OwnedEnv::new();
    let saved_from = from_env.save(from);

    let (tx, rx) = std::sync::mpsc::channel();
    if resource.send(worker::Message::Eval(code, tx)).is_err() {
        let _ = from_env.send_and_clear(&pid, |env| {
            (
                atoms::eval_reply(),
                saved_from.load(env),
                (atoms::error(), "runtime_dead"),
            )
        });
        return atoms::ok();
    }

    std::thread::spawn(move || {
        let result = match rx.recv() {
            Ok(Ok(val)) => (atoms::ok(), val),
            Ok(Err(err)) => (atoms::error(), err),
            Err(_) => (atoms::error(), "runtime_dead".to_string()),
        };
        let _ = from_env.send_and_clear(&pid, |env| {
            (atoms::eval_reply(), saved_from.load(env), result)
        });
    });

    atoms::ok()
}

#[rustler::nif(schedule = "DirtyIo")]
fn call_sync(
    resource: ResourceArc<runtime::Runtime>,
    fn_name: String,
    args_json: String,
) -> NifResult<(rustler::Atom, String)> {
    let (tx, rx) = std::sync::mpsc::channel();
    resource
        .send(worker::Message::Call(fn_name, args_json, tx))
        .map_err(|_| rustler::Error::Term(Box::new(atoms::dead_runtime())))?;

    match rx.recv() {
        Ok(Ok(val)) => Ok((atoms::ok(), val)),
        Ok(Err(err)) => Ok((atoms::error(), err)),
        Err(_) => Err(rustler::Error::Term(Box::new(atoms::dead_runtime()))),
    }
}

#[rustler::nif(schedule = "DirtyIo")]
fn load_module(
    resource: ResourceArc<runtime::Runtime>,
    name: String,
    code: String,
) -> NifResult<(rustler::Atom, String)> {
    let (tx, rx) = std::sync::mpsc::channel();
    resource
        .send(worker::Message::LoadModule(name, code, tx))
        .map_err(|_| rustler::Error::Term(Box::new(atoms::dead_runtime())))?;

    match rx.recv() {
        Ok(Ok(val)) => Ok((atoms::ok(), val)),
        Ok(Err(err)) => Ok((atoms::error(), err)),
        Err(_) => Err(rustler::Error::Term(Box::new(atoms::dead_runtime()))),
    }
}

#[rustler::nif(schedule = "DirtyIo")]
fn reset_runtime(resource: ResourceArc<runtime::Runtime>) -> NifResult<(rustler::Atom, String)> {
    let (tx, rx) = std::sync::mpsc::channel();
    resource
        .send(worker::Message::Reset(tx))
        .map_err(|_| rustler::Error::Term(Box::new(atoms::dead_runtime())))?;

    match rx.recv() {
        Ok(Ok(val)) => Ok((atoms::ok(), val)),
        Ok(Err(err)) => Ok((atoms::error(), err)),
        Err(_) => Err(rustler::Error::Term(Box::new(atoms::dead_runtime()))),
    }
}

#[rustler::nif]
fn eval_with_callbacks(
    env: Env,
    resource: ResourceArc<runtime::Runtime>,
    code: String,
    fn_names: Vec<String>,
) -> NifResult<rustler::Atom> {
    let pid = env.pid();
    let callbacks = Arc::clone(&resource.callbacks);
    resource
        .send(worker::Message::EvalWithCallbacks(code, fn_names, callbacks, pid))
        .map_err(|_| rustler::Error::Term(Box::new(atoms::dead_runtime())))?;
    Ok(atoms::ok())
}

#[rustler::nif]
fn respond_callback(
    resource: ResourceArc<runtime::Runtime>,
    callback_id: u64,
    result_json: String,
) -> rustler::Atom {
    resource.callbacks.respond(callback_id, result_json);
    atoms::ok()
}

#[rustler::nif(schedule = "DirtyIo")]
fn stop_runtime(resource: ResourceArc<runtime::Runtime>) -> rustler::Atom {
    let (tx, rx) = std::sync::mpsc::channel();
    let _ = resource.send(worker::Message::Stop(tx));
    let _ = rx.recv();
    atoms::ok()
}

rustler::init!("Elixir.QuickJSEx.Native");
