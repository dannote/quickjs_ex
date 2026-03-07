defmodule QuickJSEx.Runtime do
  @moduledoc """
  A GenServer wrapping a QuickJS-NG JavaScript runtime.

  Each runtime runs on a dedicated OS thread with its own JS context.
  Multiple runtimes can be started for concurrency.
  """
  use GenServer

  @enforce_keys [:resource]
  defstruct [:resource]

  @type t :: %__MODULE__{resource: reference()}

  @spec start_link(keyword()) :: GenServer.on_start()
  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts, Keyword.take(opts, [:name]))
  end

  @spec eval(GenServer.server(), String.t()) :: QuickJSEx.js_result()
  def eval(server, code) when is_binary(code) do
    GenServer.call(server, {:eval, code}, :infinity)
  end

  @spec call(GenServer.server(), String.t(), list()) :: QuickJSEx.js_result()
  def call(server, fn_name, args) when is_binary(fn_name) and is_list(args) do
    GenServer.call(server, {:call, fn_name, args}, :infinity)
  end

  @spec stop(GenServer.server()) :: :ok
  def stop(server) do
    GenServer.stop(server)
  end

  @impl true
  def init(_opts) do
    QuickJSEx.Native.start_runtime(self())

    receive do
      {:ok, resource} -> {:ok, %__MODULE__{resource: resource}}
      {:error, reason} -> {:stop, reason}
    after
      5_000 -> {:stop, :timeout}
    end
  end

  @impl true
  def handle_call({:eval, code}, _from, %{resource: resource} = state) do
    case QuickJSEx.Native.eval_sync(resource, code) do
      {:ok, json} -> {:reply, decode_result(json), state}
      {:error, msg} -> {:reply, {:error, msg}, state}
    end
  end

  def handle_call({:call, fn_name, args}, _from, %{resource: resource} = state) do
    args_json = Jason.encode!(args)

    case QuickJSEx.Native.call_sync(resource, fn_name, args_json) do
      {:ok, json} -> {:reply, decode_result(json), state}
      {:error, msg} -> {:reply, {:error, msg}, state}
    end
  end

  @impl true
  def terminate(_reason, %{resource: resource}) do
    QuickJSEx.Native.stop_runtime(resource)
    :ok
  end

  def terminate(_reason, _state), do: :ok

  @spec decode_result(String.t()) :: {:ok, term()} | {:error, String.t()}
  defp decode_result(json) do
    case Jason.decode(json) do
      {:ok, value} -> {:ok, value}
      {:error, _} -> {:ok, json}
    end
  end
end
