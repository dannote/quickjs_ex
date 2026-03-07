use rquickjs::{Context, Runtime, CatchResultExt, Value};

type ResultSender = std::sync::mpsc::Sender<Result<String, String>>;

pub enum Message {
    Eval(String, ResultSender),
    Call(String, String, ResultSender),
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

        let ctx = Context::full(&rt)
            .map_err(|e| format!("Failed to create QuickJS context: {e}"))?;

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
        // Try as a script first (cheaper, covers most cases including non-await code)
        let script_result = self.ctx.with(|ctx| {
            match ctx.eval::<Value, _>(code).catch(&ctx) {
                Ok(val) => Ok(value_to_json(&ctx, val)),
                Err(rquickjs::CaughtError::Exception(val)) => {
                    let msg = if val.is_object() {
                        val.as_object().get::<_, String>("message").unwrap_or_default()
                    } else {
                        String::new()
                    };
                    // Top-level await causes syntax errors in script mode.
                    // Common messages: "expecting ';'" (at the await keyword),
                    // "Unexpected token 'await'", "unexpected token"
                    if msg.contains("expecting") || msg.contains("unexpected") || msg.contains("await") {
                        Err(())
                    } else {
                        Ok(Err(format_caught_error(rquickjs::CaughtError::Exception(val))))
                    }
                }
                Err(e) => Ok(Err(format_caught_error(e))),
            }
        });

        match script_result {
            Ok(r) => r,
            Err(()) => self.eval_as_module(code),
        }
    }

    fn eval_as_module(&mut self, code: &str) -> Result<String, String> {
        let id = MODULE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let result_key = format!("__qjs_result_{id}");

        // Strategy 1: Try wrapping as expression (works for single/multi-line expressions with await)
        let expr_code = format!("globalThis.{result_key} = (\n{code}\n);\n");
        let try_expr: Result<bool, rquickjs::Error> = self.ctx.with(|ctx| {
            use rquickjs::Module;
            let module = Module::declare(ctx.clone(), format!("<module-{id}-expr>"), expr_code)
                .catch(&ctx);
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

        // Strategy 2: Multi-statement code — put directly as module body.
        // Split into statements: all but last run as-is, last one captures result.
        let trimmed = code.trim();
        let module_code = if let Some(pos) = trimmed.rfind('\n') {
            let (setup, last_line) = trimmed.split_at(pos);
            let last_line = last_line.trim();
            // If last line is a pure expression (starts with await, or is a function call, etc.)
            format!("{setup}\nglobalThis.{result_key} = {last_line};\n")
        } else {
            format!("globalThis.{result_key} = {trimmed};\n")
        };

        let id2 = MODULE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let declare_result = self.ctx.with(|ctx| {
            use rquickjs::Module;
            let module = Module::declare(ctx.clone(), format!("<module-{id2}>"), module_code)
                .catch(&ctx)
                .map_err(format_caught_error)?;
            module.eval().catch(&ctx).map_err(format_caught_error)?;
            Ok::<_, String>(())
        });

        if let Err(e) = declare_result {
            return Err(e);
        }

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

    fn call(&self, fn_name: &str, args_json: &str) -> Result<String, String> {
        self.ctx.with(|ctx| {
            let code = format!(
                r#"JSON.stringify({fn_name}.apply(null, {args_json}))"#
            );
            let val: Value = ctx
                .eval(code.as_bytes().to_vec())
                .catch(&ctx)
                .map_err(format_caught_error)?;

            if let Some(s) = val.as_string() {
                let json_str = s.to_string().map_err(|e| format!("String conversion: {e}"))?;
                Ok(json_str)
            } else {
                Ok("null".to_string())
            }
        })
    }
}

fn value_to_json<'js>(ctx: &rquickjs::Ctx<'js>, val: Value<'js>) -> Result<String, String> {
    if val.is_undefined() || val.is_null() {
        return Ok("null".to_string());
    }

    if let Some(s) = val.as_string() {
        return s.to_string().map_err(|e| format!("String conversion error: {e}"));
    }

    let json: rquickjs::String = ctx
        .json_stringify(val)
        .map_err(|e| format!("JSON stringify error: {e}"))?
        .ok_or_else(|| "Value is not JSON-serializable".to_string())?;

    json.to_string().map_err(|e| format!("String conversion error: {e}"))
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
