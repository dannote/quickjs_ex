use rquickjs::{CatchResultExt, Context, Runtime, Value};

type ResultSender = std::sync::mpsc::Sender<Result<String, String>>;

pub enum Message {
    Eval(String, ResultSender),
    Call(String, String, ResultSender),
    LoadModule(String, String, ResultSender),
    Stop(std::sync::mpsc::Sender<()>),
}

static MODULE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub struct Worker {
    rt: Runtime,
    ctx: Context,
}

impl Worker {
    pub fn new() -> Result<Self, String> {
        let rt = Runtime::new().map_err(|e| format!("Failed to create QuickJS runtime: {e}"))?;
        rt.set_memory_limit(256 * 1024 * 1024);
        rt.set_max_stack_size(1024 * 1024);
        rt.set_gc_threshold(4 * 1024 * 1024);

        let ctx =
            Context::full(&rt).map_err(|e| format!("Failed to create QuickJS context: {e}"))?;

        ctx.with(|ctx| {
            ctx.eval::<(), _>(
                r#"
                globalThis.console = {
                    log: function() {},
                    warn: function() {},
                    error: function() {},
                    info: function() {},
                    debug: function() {},
                };
                "#,
            )
            .catch(&ctx)
            .map_err(|e| format!("Failed to install console stub: {e:?}"))
        })?;

        Ok(Self { rt, ctx })
    }

    pub fn run(&mut self, receiver: std::sync::mpsc::Receiver<Message>) {
        while let Ok(msg) = receiver.recv() {
            match msg {
                Message::Eval(code, tx) => {
                    let result = self.eval(&code);
                    let _ = tx.send(result);
                }
                Message::Call(fn_name, args_json, tx) => {
                    let result = self.call(&fn_name, &args_json);
                    let _ = tx.send(result);
                }
                Message::LoadModule(name, code, tx) => {
                    let result = self.load_module(&name, &code);
                    let _ = tx.send(result);
                }
                Message::Stop(tx) => {
                    let _ = tx.send(());
                    break;
                }
            }
        }
    }

    fn drain_jobs(&self) {
        loop {
            match self.rt.execute_pending_job() {
                Ok(false) => break,
                Ok(true) => continue,
                Err(_) => break,
            }
        }
    }

    fn eval(&mut self, code: &str) -> Result<String, String> {
        let script_result = self
            .ctx
            .with(|ctx| match ctx.eval::<Value, _>(code).catch(&ctx) {
                Ok(val) => Ok(value_to_json(&ctx, val)),
                Err(rquickjs::CaughtError::Exception(val)) => {
                    let msg = if val.is_object() {
                        val.as_object()
                            .get::<_, String>("message")
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    if msg.contains("expecting")
                        || msg.contains("unexpected")
                        || msg.contains("await")
                    {
                        Err(())
                    } else {
                        Ok(Err(format_caught_error(rquickjs::CaughtError::Exception(
                            val,
                        ))))
                    }
                }
                Err(e) => Ok(Err(format_caught_error(e))),
            });

        match script_result {
            Ok(r) => r,
            Err(()) => self.eval_as_module(code),
        }
    }

    fn eval_as_module(&mut self, code: &str) -> Result<String, String> {
        let id = MODULE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let result_key = format!("__qjs_result_{id}");

        let expr_code = format!("globalThis.{result_key} = (\n{code}\n);\n");
        let try_expr: Result<bool, rquickjs::Error> = self.ctx.with(|ctx| {
            use rquickjs::Module;
            let module =
                Module::declare(ctx.clone(), format!("<module-{id}-expr>"), expr_code).catch(&ctx);
            match module {
                Ok(m) => match m.eval().catch(&ctx) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                },
                Err(_) => Ok(false),
            }
        });

        if matches!(try_expr, Ok(true)) {
            self.drain_jobs();
            return self.ctx.with(|ctx| {
                let global = ctx.globals();
                let val: Value = global
                    .get(&*result_key)
                    .unwrap_or(Value::new_undefined(ctx.clone()));
                let _ = global.remove(&*result_key);
                value_to_json(&ctx, val)
            });
        }

        let trimmed = code.trim();
        let module_code = if let Some(pos) = trimmed.rfind('\n') {
            let (setup, last_line) = trimmed.split_at(pos);
            let last_line = last_line.trim();
            format!("{setup}\nglobalThis.{result_key} = {last_line};\n")
        } else {
            format!("globalThis.{result_key} = {trimmed};\n")
        };

        let id2 = MODULE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.ctx.with(|ctx| {
            use rquickjs::Module;
            let module = Module::declare(ctx.clone(), format!("<module-{id2}>"), module_code)
                .catch(&ctx)
                .map_err(format_caught_error)?;
            module.eval().catch(&ctx).map_err(format_caught_error)?;
            Ok::<_, String>(())
        })?;

        self.drain_jobs();

        self.ctx.with(|ctx| {
            let global = ctx.globals();
            let val: Value = global
                .get(&*result_key)
                .unwrap_or(Value::new_undefined(ctx.clone()));
            let _ = global.remove(&*result_key);
            value_to_json(&ctx, val)
        })
    }

    fn load_module(&mut self, name: &str, code: &str) -> Result<String, String> {
        self.ctx.with(|ctx| {
            use rquickjs::Module;
            let module = Module::declare(ctx.clone(), name, code.as_bytes().to_vec())
                .catch(&ctx)
                .map_err(format_caught_error)?;
            let (module, _) = module.eval().catch(&ctx).map_err(format_caught_error)?;

            let namespace: rquickjs::Object = module.namespace().map_err(|e| format!("{e}"))?;
            let global = ctx.globals();

            let keys: Vec<String> = namespace.keys::<String>().filter_map(|k| k.ok()).collect();

            for key in &keys {
                let val: Value = namespace
                    .get(key.as_str())
                    .unwrap_or(Value::new_undefined(ctx.clone()));
                global
                    .set(key.as_str(), val)
                    .map_err(|e| format!("Failed to set global {key}: {e}"))?;
            }

            Ok::<_, String>(())
        })?;

        self.drain_jobs();
        Ok("ok".to_string())
    }

    /// Call a global function. If it returns a Promise, resolve it via
    /// a globalThis trampoline: install .then/.catch handlers, drain the
    /// job queue *outside* ctx.with() (avoiding the runtime lock deadlock),
    /// then read the settled value.
    fn call(&mut self, fn_name: &str, args_json: &str) -> Result<String, String> {
        let id = MODULE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let result_key = format!("__qjs_call_{id}");
        let error_key = format!("__qjs_call_err_{id}");
        let settled_key = format!("__qjs_call_settled_{id}");

        // Phase 1: invoke the function and install promise handlers (if needed)
        let is_promise = self.ctx.with(|ctx| {
            // Call the function and check if result is thenable
            let code = format!(
                r#"(function() {{
                    var __r = {fn_name}.apply(null, {args_json});
                    if (__r && typeof __r === 'object' && typeof __r.then === 'function') {{
                        __r.then(
                            function(v) {{ globalThis.{result_key} = v; globalThis.{settled_key} = true; }},
                            function(e) {{ globalThis.{error_key} = e instanceof Error ? e.message : String(e); globalThis.{settled_key} = true; }}
                        );
                        return true;
                    }} else {{
                        globalThis.{result_key} = __r;
                        globalThis.{settled_key} = true;
                        return false;
                    }}
                }})()"#
            );
            let val: Value = ctx
                .eval(code.as_bytes().to_vec())
                .catch(&ctx)
                .map_err(format_caught_error)?;

            let is_promise = val.as_bool().unwrap_or(false);
            Ok::<bool, String>(is_promise)
        })?;

        // Phase 2: drain pending jobs OUTSIDE ctx.with() to avoid deadlock
        if is_promise {
            self.drain_jobs();
        }

        // Phase 3: read the settled result
        self.ctx.with(|ctx| {
            let global = ctx.globals();

            let has_error: bool = global
                .get::<_, Value>(&*error_key)
                .map(|v| !v.is_undefined())
                .unwrap_or(false);

            let result = if has_error {
                let err_msg: String = global.get(&*error_key).unwrap_or_default();
                Err(err_msg)
            } else {
                let val: Value = global
                    .get(&*result_key)
                    .unwrap_or(Value::new_undefined(ctx.clone()));
                value_to_json(&ctx, val)
            };

            let _ = global.remove(&*result_key);
            let _ = global.remove(&*error_key);
            let _ = global.remove(&*settled_key);

            result
        })
    }
}

fn value_to_json<'js>(ctx: &rquickjs::Ctx<'js>, val: Value<'js>) -> Result<String, String> {
    if val.is_undefined() || val.is_null() {
        return Ok("null".to_string());
    }

    if let Some(s) = val.as_string() {
        return s
            .to_string()
            .map_err(|e| format!("String conversion error: {e}"));
    }

    let json: rquickjs::String = ctx
        .json_stringify(val)
        .map_err(|e| format!("JSON stringify error: {e}"))?
        .ok_or_else(|| "Value is not JSON-serializable".to_string())?;

    json.to_string()
        .map_err(|e| format!("String conversion error: {e}"))
}

fn format_caught_error(err: rquickjs::CaughtError<'_>) -> String {
    match err {
        rquickjs::CaughtError::Exception(val) => {
            if val.is_object() {
                let obj = val.as_object();
                let message: String = obj.get("message").unwrap_or_default();
                let stack: String = obj.get("stack").unwrap_or_default();
                if stack.is_empty() {
                    message
                } else {
                    format!("{message}\n{stack}")
                }
            } else {
                format!("{val:?}")
            }
        }
        rquickjs::CaughtError::Value(val) => format!("Thrown value: {val:?}"),
        rquickjs::CaughtError::Error(e) => format!("{e}"),
    }
}
