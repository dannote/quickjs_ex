# Changelog

## 0.1.1

- `eval/2` now handles code with `export` statements — exports are automatically promoted to `globalThis`, matching `load_module/3` behavior
- Built-in `atob` and `btoa` (base64 encoding/decoding) available in all contexts
- Built-in `Buffer` polyfill (`from`, `alloc`, `concat`, `isBuffer`, `toString` with utf-8/base64/binary encodings)

## 0.1.0

Initial release.

- QuickJS-NG engine embedded via Rustler NIF (rquickjs 0.11)
- `eval/2` — evaluate JavaScript with automatic top-level `await` support
- `call/3` — call global JS functions with Elixir arguments; Promises are automatically awaited
- `load_module/3` — load ES modules, promoting named exports to `globalThis` for `call/3`
- `reset/1` — replace JS context without restarting the OS thread
- `start/1` / `stop/1` — lifecycle management
- `QuickJSEx.Runtime` GenServer for supervised usage
- ES2023+ support: async/await, Promises, Proxy, Map/Set, destructuring, modules
- Dedicated OS thread per runtime — no BEAM scheduler contention
- Persistent `globalThis` state across evaluations
- Runtime isolation — separate runtimes share no state
- Console stub (`console.log` etc.) installed by default
- 256 MB memory limit, 1 MB stack, 4 MB GC threshold defaults
