# QuickJSEx

[![Hex.pm](https://img.shields.io/hexpm/v/quickjs_ex.svg)](https://hex.pm/packages/quickjs_ex)
[![Docs](https://img.shields.io/badge/hex-docs-blue.svg)](https://hexdocs.pm/quickjs_ex)

Embedded [QuickJS-NG](https://quickjs-ng.github.io/quickjs/) JavaScript engine for Elixir via [Rustler](https://github.com/rusterlium/rustler) NIF.

- **No external runtime** — no Node.js, Bun, or Deno required
- **In-process** — runs inside the BEAM via a 1.6 MB NIF
- **ES2023+** — full async/await, Promises, Proxy, Map/Set, destructuring, modules
- **Isolated** — each runtime runs on a dedicated OS thread with its own JS context
- **Persistent state** — globals survive across evaluations within a runtime

## Installation

Add `quickjs_ex` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:quickjs_ex, "~> 0.1.0"}
  ]
end
```

Requires a Rust toolchain for compilation. Install via [rustup](https://rustup.rs/).

## Usage

```elixir
# Start a runtime
{:ok, rt} = QuickJSEx.start()

# Evaluate JavaScript
{:ok, 3} = QuickJSEx.eval(rt, "1 + 2")
{:ok, "hello"} = QuickJSEx.eval(rt, "'hello'")
{:ok, %{"a" => 1}} = QuickJSEx.eval(rt, "({a: 1})")

# State persists across calls
{:ok, _} = QuickJSEx.eval(rt, "globalThis.counter = 0")
{:ok, _} = QuickJSEx.eval(rt, "counter += 1")
{:ok, 1} = QuickJSEx.eval(rt, "counter")

# Async/await works
{:ok, [1, 2, 3]} = QuickJSEx.eval(rt, """
  await Promise.all([
    Promise.resolve(1),
    Promise.resolve(2),
    Promise.resolve(3)
  ])
""")

# Call global functions (sync and async)
{:ok, _} = QuickJSEx.eval(rt, "async function greet(name) { return 'hi ' + name; }")
{:ok, "hi world"} = QuickJSEx.call(rt, "greet", ["world"])

# Stop when done
QuickJSEx.stop(rt)
```

## ES Module Loading

Load ES modules built with Vite, esbuild, or Rollup. Named exports are
promoted to `globalThis`, making them callable with `call/3`:

```elixir
{:ok, rt} = QuickJSEx.start()

:ok = QuickJSEx.load_module(rt, "math", """
  export function add(a, b) { return a + b; }
  export const PI = 3.14159;
""")

{:ok, 5} = QuickJSEx.call(rt, "add", [2, 3])
{:ok, 3.14159} = QuickJSEx.eval(rt, "PI")
```

## SSR Usage (e.g., with LiveVue)

```elixir
{:ok, rt} = QuickJSEx.start()
:ok = QuickJSEx.load_module(rt, "server", File.read!("priv/static/server.js"))
{:ok, html} = QuickJSEx.call(rt, "render", ["MyComponent", %{count: 0}, %{}])
```

Async `render` functions (returning Promises) are automatically awaited.

## Supervision

```elixir
children = [
  {QuickJSEx.Runtime, name: MyApp.JS}
]

Supervisor.start_link(children, strategy: :one_for_one)

# Then use the named process:
{:ok, result} = QuickJSEx.eval(MyApp.JS, "1 + 2")
```

## Architecture

Each `QuickJSEx.Runtime` spawns a dedicated OS thread running a QuickJS-NG context. Communication between the BEAM and the JS thread uses `std::sync::mpsc` channels. JS execution never blocks BEAM schedulers.

```
┌──────────────────────┐     mpsc channel     ┌─────────────────────┐
│  BEAM Process        │ ──────────────────▶   │  OS Thread          │
│  (GenServer)         │                       │  QuickJS Runtime    │
│                      │ ◀──────────────────   │  + Context          │
└──────────────────────┘     result channel    └─────────────────────┘
```

## License

[MIT](LICENSE)
