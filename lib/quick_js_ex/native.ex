defmodule QuickJSEx.Native do
  @moduledoc false
  use Rustler, otp_app: :quickjs_ex, crate: "quickjs_ex_nif"

  @spec start_runtime(pid()) :: :ok
  def start_runtime(_pid), do: :erlang.nif_error(:nif_not_loaded)

  @spec eval_sync(reference(), String.t()) :: {:ok, String.t()} | {:error, String.t()}
  def eval_sync(_runtime, _code), do: :erlang.nif_error(:nif_not_loaded)

  @spec eval_async(term(), reference(), String.t()) :: :ok
  def eval_async(_from, _runtime, _code), do: :erlang.nif_error(:nif_not_loaded)

  @spec call_sync(reference(), String.t(), String.t()) :: {:ok, String.t()} | {:error, String.t()}
  def call_sync(_runtime, _fn_name, _args_json), do: :erlang.nif_error(:nif_not_loaded)

  @spec load_module(reference(), String.t(), String.t()) ::
          {:ok, String.t()} | {:error, String.t()}
  def load_module(_runtime, _name, _code), do: :erlang.nif_error(:nif_not_loaded)

  @spec stop_runtime(reference()) :: :ok
  def stop_runtime(_runtime), do: :erlang.nif_error(:nif_not_loaded)
end
