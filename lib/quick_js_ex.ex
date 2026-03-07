defmodule QuickJSEx do
  @moduledoc """
  Embedded QuickJS-NG JavaScript engine for Elixir.

  QuickJSEx embeds the QuickJS-NG runtime via a Rustler NIF, providing
  in-process JavaScript execution with no external runtime dependencies
  (no Node.js, Bun, or Deno required).

  ## Quick Start

      {:ok, rt} = QuickJSEx.start()
      {:ok, 3} = QuickJSEx.eval(rt, "1 + 2")
      QuickJSEx.stop(rt)

  ## With a Supervisor

      children = [
        {QuickJSEx.Runtime, name: MyApp.JS}
      ]

      # Then:
      {:ok, result} = QuickJSEx.eval(MyApp.JS, "JSON.stringify({a: 1})")

  ## SSR Usage

      {:ok, rt} = QuickJSEx.start()
      :ok = QuickJSEx.load_module(rt, "server", File.read!("priv/static/server.js"))
      {:ok, html} = QuickJSEx.call(rt, "render", ["MyComponent", %{count: 0}, %{}])
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
end
