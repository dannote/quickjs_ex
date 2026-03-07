defmodule QuickJSEx.Native do
  @moduledoc false

  version = Mix.Project.config()[:version]

  use RustlerPrecompiled,
    otp_app: :quickjs_ex,
    crate: "quickjs_ex_nif",
    base_url: "https://github.com/dannote/quickjs_ex/releases/download/v#{version}",
    force_build: System.get_env("QUICKJS_EX_BUILD") in ["1", "true"],
    targets: ~w(
      aarch64-apple-darwin
      aarch64-unknown-linux-gnu
      aarch64-unknown-linux-musl
      x86_64-apple-darwin
      x86_64-unknown-linux-gnu
      x86_64-unknown-linux-musl
      x86_64-pc-windows-gnu
      x86_64-pc-windows-msvc
    ),
    nif_versions: ["2.15"],
    version: version

  @spec start_runtime(pid(), boolean()) :: :ok
  def start_runtime(_pid, _browser_stubs), do: :erlang.nif_error(:nif_not_loaded)

  @spec eval_sync(reference(), String.t()) :: {:ok, String.t()} | {:error, String.t()}
  def eval_sync(_runtime, _code), do: :erlang.nif_error(:nif_not_loaded)

  @spec eval_async(term(), reference(), String.t()) :: :ok
  def eval_async(_from, _runtime, _code), do: :erlang.nif_error(:nif_not_loaded)

  @spec call_sync(reference(), String.t(), String.t()) :: {:ok, String.t()} | {:error, String.t()}
  def call_sync(_runtime, _fn_name, _args_json), do: :erlang.nif_error(:nif_not_loaded)

  @spec load_module(reference(), String.t(), String.t()) ::
          {:ok, String.t()} | {:error, String.t()}
  def load_module(_runtime, _name, _code), do: :erlang.nif_error(:nif_not_loaded)

  @spec reset_runtime(reference()) :: {:ok, String.t()} | {:error, String.t()}
  def reset_runtime(_runtime), do: :erlang.nif_error(:nif_not_loaded)

  @spec stop_runtime(reference()) :: :ok
  def stop_runtime(_runtime), do: :erlang.nif_error(:nif_not_loaded)
end
