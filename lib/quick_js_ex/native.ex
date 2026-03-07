defmodule QuickJSEx.Native do
  use Rustler, otp_app: :quickjs_ex, crate: "quickjs_ex_nif"

  def start_runtime(_pid), do: :erlang.nif_error(:nif_not_loaded)
  def eval_sync(_runtime, _code), do: :erlang.nif_error(:nif_not_loaded)
  def eval_async(_from, _runtime, _code), do: :erlang.nif_error(:nif_not_loaded)
  def call_sync(_runtime, _fn_name, _args_json), do: :erlang.nif_error(:nif_not_loaded)
  def stop_runtime(_runtime), do: :erlang.nif_error(:nif_not_loaded)
end
