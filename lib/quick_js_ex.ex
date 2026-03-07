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
      {:ok, _} = QuickJSEx.eval(rt, File.read!("priv/static/server.js"))
      {:ok, html} = QuickJSEx.call(rt, "render", ["MyComponent", %{count: 0}, %{}])
  """

  @doc "Start a new JavaScript runtime."
  def start(opts \\ []) do
    QuickJSEx.Runtime.start_link(opts)
  end

  @doc "Evaluate JavaScript code and return the result."
  def eval(runtime, code) do
    QuickJSEx.Runtime.eval(runtime, code)
  end

  @doc "Call a global JavaScript function by name with the given arguments."
  def call(runtime, fn_name, args \\ []) do
    QuickJSEx.Runtime.call(runtime, fn_name, args)
  end

  @doc "Stop a runtime."
  def stop(runtime) do
    QuickJSEx.Runtime.stop(runtime)
  end
end
