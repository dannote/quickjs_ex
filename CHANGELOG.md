# Changelog

## 0.1.0

Initial release.

- QuickJS-NG engine embedded via Rustler NIF (rquickjs 0.11)
- `eval/2` — evaluate JavaScript with automatic top-level `await` support
- `call/3` — call global JS functions with Elixir arguments; Promises are automatically awaited
- `load_module/3` — load ES modules, promoting named exports to `globalThis` for `call/3`
- `start/1` / `stop/1` — lifecycle management
- `QuickJSEx.Runtime` GenServer for supervised usage
- ES2023+ support: async/await, Promises, Proxy, Map/Set, destructuring, modules
- Dedicated OS thread per runtime — no BEAM scheduler contention
- Persistent `globalThis` state across evaluations
- Runtime isolation — separate runtimes share no state
- Console stub (`console.log` etc.) installed by default
- 256 MB memory limit, 1 MB stack, 4 MB GC threshold defaults
