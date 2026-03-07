# Changelog

## 0.3.0

- Precompiled NIF binaries for macOS (aarch64, x86_64), Linux (aarch64, x86_64, gnu and musl), and Windows (x86_64, gnu and msvc) — Rust toolchain no longer required for most users
- Uses `rustler_precompiled`; set `QUICKJS_EX_BUILD=true` to force local compilation

## 0.2.0

- `eval/2` now handles code with `export` statements — exports are automatically promoted to `globalThis`, matching `load_module/3` behavior
- Built-in `atob`/`btoa` (base64) and `TextEncoder`/`TextDecoder` (UTF-8) available in all contexts
- Full `Buffer` polyfill: `from`/`alloc`/`allocUnsafe`/`concat`/`compare`/`isBuffer`/`isEncoding`/`byteLength`, all encodings (utf-8, base64, base64url, hex, latin1, ascii, utf16le), read/write integer methods, `slice`/`copy`/`indexOf`/`fill`/`write`/`equals`/`compare`/`toJSON`
- `console` stub (no-op) — prevents `console is not defined` errors
- New `:browser_stubs` option for `start/1` — installs `window`, `document`, `navigator`, `localStorage`, `sessionStorage`, `process`, `location`, `matchMedia`, `MutationObserver`, `ResizeObserver`, `IntersectionObserver`, `Event`/`CustomEvent`, `requestAnimationFrame`, `getComputedStyle`

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
