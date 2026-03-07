use base64::{engine::general_purpose::STANDARD, Engine};
use rquickjs::{CatchResultExt, Context, Function, Runtime, Value};

type ResultSender = std::sync::mpsc::Sender<Result<String, String>>;

pub enum Message {
    Eval(String, ResultSender),
    Call(String, String, ResultSender),
    LoadModule(String, String, ResultSender),
    Reset(ResultSender),
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

        let ctx = Self::create_context(&rt)?;
        Ok(Self { rt, ctx })
    }

    fn create_context(rt: &Runtime) -> Result<Context, String> {
        let ctx =
            Context::full(rt).map_err(|e| format!("Failed to create QuickJS context: {e}"))?;

        ctx.with(|ctx| {
            let globals = ctx.globals();

            let btoa = Function::new(ctx.clone(), |input: String| -> rquickjs::Result<String> {
                Ok(STANDARD.encode(input.as_bytes()))
            })
            .map_err(|e| format!("Failed to create btoa: {e}"))?;
            globals
                .set("btoa", btoa)
                .map_err(|e| format!("Failed to set btoa: {e}"))?;

            let atob = Function::new(
                ctx.clone(),
                |ctx: rquickjs::Ctx<'_>, input: String| -> rquickjs::Result<String> {
                    match STANDARD.decode(&input) {
                        Ok(bytes) => Ok(bytes.iter().map(|&b| b as char).collect()),
                        Err(e) => {
                            ctx.throw(
                                rquickjs::String::from_str(ctx.clone(), &format!("Invalid base64: {e}"))?.into(),
                            );
                            Err(rquickjs::Error::Exception)
                        }
                    }
                },
            )
            .map_err(|e| format!("Failed to create atob: {e}"))?;
            globals
                .set("atob", atob)
                .map_err(|e| format!("Failed to set atob: {e}"))?;

            // Buffer polyfill — covers the subset used by Vue SSR and most npm packages
            ctx.eval::<(), _>(
                r#"
                (function() {
                    function toBytes(str) {
                        var arr = [];
                        for (var i = 0; i < str.length; i++) {
                            var c = str.charCodeAt(i);
                            if (c < 0x80) {
                                arr.push(c);
                            } else if (c < 0x800) {
                                arr.push(0xc0 | (c >> 6), 0x80 | (c & 0x3f));
                            } else if (c >= 0xd800 && c <= 0xdbff && i + 1 < str.length) {
                                var c2 = str.charCodeAt(++i);
                                var cp = ((c - 0xd800) << 10) + (c2 - 0xdc00) + 0x10000;
                                arr.push(0xf0 | (cp >> 18), 0x80 | ((cp >> 12) & 0x3f),
                                         0x80 | ((cp >> 6) & 0x3f), 0x80 | (cp & 0x3f));
                            } else {
                                arr.push(0xe0 | (c >> 12), 0x80 | ((c >> 6) & 0x3f), 0x80 | (c & 0x3f));
                            }
                        }
                        return new Uint8Array(arr);
                    }
                    function fromBytes(bytes) {
                        var str = '', i = 0;
                        while (i < bytes.length) {
                            var b = bytes[i++];
                            if (b < 0x80) { str += String.fromCharCode(b); }
                            else if (b < 0xe0) { str += String.fromCharCode(((b & 0x1f) << 6) | (bytes[i++] & 0x3f)); }
                            else if (b < 0xf0) { str += String.fromCharCode(((b & 0x0f) << 12) | ((bytes[i++] & 0x3f) << 6) | (bytes[i++] & 0x3f)); }
                            else {
                                var cp = ((b & 0x07) << 18) | ((bytes[i++] & 0x3f) << 12) | ((bytes[i++] & 0x3f) << 6) | (bytes[i++] & 0x3f);
                                cp -= 0x10000;
                                str += String.fromCharCode(0xd800 + (cp >> 10), 0xdc00 + (cp & 0x3ff));
                            }
                        }
                        return str;
                    }
                    function makeBuffer(bytes, source) {
                        bytes._source = source;
                        bytes.toString = function(enc) {
                            if (enc === 'base64') {
                                var s = '';
                                for (var i = 0; i < this.length; i++) s += String.fromCharCode(this[i]);
                                return btoa(s);
                            }
                            if (!enc || enc === 'utf-8' || enc === 'utf8') return fromBytes(this);
                            if (enc === 'binary' || enc === 'latin1') {
                                var s = '';
                                for (var i = 0; i < this.length; i++) s += String.fromCharCode(this[i]);
                                return s;
                            }
                            return fromBytes(this);
                        };
                        return bytes;
                    }
                    globalThis.Buffer = {
                        from: function(input, encoding) {
                            if (typeof input === 'string') {
                                if (encoding === 'base64') {
                                    var str = atob(input);
                                    var bytes = new Uint8Array(str.length);
                                    for (var i = 0; i < str.length; i++) bytes[i] = str.charCodeAt(i);
                                    return makeBuffer(bytes, input);
                                }
                                return makeBuffer(toBytes(input), input);
                            }
                            if (Array.isArray(input)) return makeBuffer(new Uint8Array(input));
                            if (input instanceof Uint8Array) return makeBuffer(input);
                            if (input instanceof ArrayBuffer) return makeBuffer(new Uint8Array(input));
                            throw new TypeError('The first argument must be a string, Buffer, ArrayBuffer, or Array');
                        },
                        alloc: function(size) { return makeBuffer(new Uint8Array(size)); },
                        isBuffer: function(obj) { return obj instanceof Uint8Array && typeof obj.toString === 'function' && obj._source !== undefined; },
                        concat: function(list) {
                            var total = 0;
                            for (var i = 0; i < list.length; i++) total += list[i].length;
                            var result = new Uint8Array(total);
                            var offset = 0;
                            for (var i = 0; i < list.length; i++) { result.set(list[i], offset); offset += list[i].length; }
                            return makeBuffer(result);
                        }
                    };
                })();"#,
            )
            .catch(&ctx)
            .map_err(|e| format!("Failed to install Buffer: {e:?}"))?;

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
            .map_err(|e| format!("Failed to install globals: {e:?}"))
        })?;

        Ok(ctx)
    }

    fn reset(&mut self) -> Result<String, String> {
        self.ctx = Self::create_context(&self.rt)?;
        Ok("ok".to_string())
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
                Message::Reset(tx) => {
                    let result = self.reset();
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
                        || msg.contains("unsupported")
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
        let has_export = code.contains("export ");

        if has_export {
            // Module with exports: evaluate and promote exports to globalThis
            let name = format!("<eval-module-{id}>");
            self.load_module(&name, code)?;
            Ok("null".to_string())
        } else {
            // Module without exports (e.g. top-level await): capture last expression
            let result_key = format!("__qjs_result_{id}");

            let expr_code = format!("globalThis.{result_key} = (\n{code}\n);\n");
            let try_expr: Result<bool, rquickjs::Error> = self.ctx.with(|ctx| {
                use rquickjs::Module;
                let module = Module::declare(
                    ctx.clone(),
                    format!("<module-{id}-expr>"),
                    expr_code,
                )
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
                let module =
                    Module::declare(ctx.clone(), format!("<module-{id2}>"), module_code)
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
