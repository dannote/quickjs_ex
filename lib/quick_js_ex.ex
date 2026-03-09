defmodule QuickJSEx do
  @moduledoc """
  Embedded QuickJS-NG JavaScript engine for Elixir.

  QuickJSEx embeds the QuickJS-NG runtime via a Rustler NIF, providing
  in-process JavaScript execution with no external runtime dependencies
  such as Node.js, Bun, or Deno.

  Precompiled NIFs are available for supported macOS, Linux, and Windows
  targets, so a Rust toolchain is not required for most installations.

  ## Quick Start

      {:ok, rt} = QuickJSEx.start()
      {:ok, 3} = QuickJSEx.eval(rt, "1 + 2")
      QuickJSEx.stop(rt)

  ## Async JavaScript

      {:ok, rt} = QuickJSEx.start()

      {:ok, [1, 2, 3]} =
        QuickJSEx.eval(rt, "await Promise.all([Promise.resolve(1), Promise.resolve(2), Promise.resolve(3)])")

  `call/3` automatically awaits Promise-returning functions.

  ## ES Modules and Exports

      {:ok, rt} = QuickJSEx.start()

      :ok = QuickJSEx.load_module(rt, "math", "export function add(a, b) { return a + b; }\nexport const PI = 3.14159;")

      {:ok, 5} = QuickJSEx.call(rt, "add", [2, 3])
      {:ok, 3.14159} = QuickJSEx.eval(rt, "PI")

  `eval/2` also supports code with `export` statements and promotes named
  exports to `globalThis`, matching `load_module/3` behavior.

  ## SSR Usage

      {:ok, rt} = QuickJSEx.start(browser_stubs: true)
      :ok = QuickJSEx.load_module(rt, "server", File.read!("priv/static/server.js"))
      {:ok, html} = QuickJSEx.call(rt, "render", ["MyComponent", %{count: 0}, %{}])

  ## Resetting a Runtime

      {:ok, rt} = QuickJSEx.start()
      {:ok, _} = QuickJSEx.eval(rt, "globalThis.answer = 42")
      :ok = QuickJSEx.reset(rt)
      {:error, _reason} = QuickJSEx.eval(rt, "answer")

  ## With a Supervisor

      children = [
        {QuickJSEx.Runtime, name: MyApp.JS, browser_stubs: true}
      ]

      {:ok, 3} = QuickJSEx.eval(MyApp.JS, "1 + 2")
  """

  @type runtime :: GenServer.server()
  @type js_result :: {:ok, term()} | {:error, String.t()}

  @doc """
  Start a new JavaScript runtime.

  ## Options

    * `:name` — GenServer name for the runtime
    * `:browser_stubs` — when `true`, installs browser environment stubs
      (`window`, `document`, `navigator`, `localStorage`, `process`, etc.)
      for running SSR bundles and npm packages that reference browser globals.
      Defaults to `false`.

  """
  @spec start(keyword()) :: GenServer.on_start()
  def start(opts \\ []) do
    QuickJSEx.Runtime.start_link(opts)
  end

  @doc "Evaluate JavaScript code and return the result."
  @spec eval(runtime(), String.t()) :: js_result()
  def eval(runtime, code) do
    QuickJSEx.Runtime.eval(runtime, code)
  end

  @doc """
  Call a global JavaScript function by name with the given arguments.

  Handles both sync and async (Promise-returning) functions — Promises are
  automatically awaited before the result is returned.
  """
  @spec call(runtime(), String.t(), list()) :: js_result()
  def call(runtime, fn_name, args \\ []) do
    QuickJSEx.Runtime.call(runtime, fn_name, args)
  end

  @doc """
  Load an ES module into the runtime.

  The module's named exports are promoted to `globalThis`, making them
  available for `call/3`. This is the primary way to load SSR bundles
  built with `vite build --ssr`.

      :ok = QuickJSEx.load_module(rt, "server", File.read!("priv/static/server.js"))
      {:ok, html} = QuickJSEx.call(rt, "render", ["MyComponent", props, slots])
  """
  @spec load_module(runtime(), String.t(), String.t()) :: :ok | {:error, String.t()}
  def load_module(runtime, name, code) do
    QuickJSEx.Runtime.load_module(runtime, name, code)
  end

  @doc """
  Reset the runtime to a fresh state.

  Replaces the JS context with a new one, clearing all global state,
  loaded modules, and function definitions. The underlying OS thread
  and runtime are reused.
  """
  @spec reset(runtime()) :: :ok | {:error, String.t()}
  def reset(runtime) do
    QuickJSEx.Runtime.reset(runtime)
  end

  @doc "Stop a runtime."
  @spec stop(runtime()) :: :ok
  def stop(runtime) do
    QuickJSEx.Runtime.stop(runtime)
  end

  @doc false
  def start_resource(opts \\ []) do
    browser_stubs = Keyword.get(opts, :browser_stubs, false)
    QuickJSEx.Native.start_runtime(self(), browser_stubs)

    receive do
      {:ok, resource} -> {:ok, resource}
      {:error, reason} -> {:error, reason}
    after
      5_000 -> {:error, :timeout}
    end
  end

  @doc false
  def eval_resource(resource, code) do
    case QuickJSEx.Native.eval_sync(resource, code) do
      {:ok, json} -> {:ok, decode_result(json)}
      {:error, msg} -> {:error, msg}
    end
  end

  @doc false
  def eval_with_callbacks(resource, code, fn_names) do
    QuickJSEx.Native.eval_with_callbacks(resource, code, fn_names)
  end

  @doc false
  def respond_callback(resource, callback_id, result_json) do
    QuickJSEx.Native.respond_callback(resource, callback_id, result_json)
  end

  @doc false
  def stop_resource(resource) do
    QuickJSEx.Native.stop_runtime(resource)
  end

  defp decode_result(json) do
    case Jason.decode(json) do
      {:ok, value} -> value
      {:error, _} -> json
    end
  end
end
