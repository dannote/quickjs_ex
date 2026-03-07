defmodule QuickJSEx.MixProject do
  use Mix.Project

  @version "0.1.0"
  @source_url "https://github.com/nicolo-valian/quickjs_ex"

  def project do
    [
      app: :quickjs_ex,
      version: @version,
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      package: package(),
      description: "Embedded QuickJS-NG JavaScript engine for Elixir via Rustler NIF",
      source_url: @source_url,
      docs: docs()
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:rustler, "~> 0.36"},
      {:jason, "~> 1.4"},
      {:ex_doc, "~> 0.31", only: :dev, runtime: false}
    ]
  end

  defp package do
    [
      licenses: ["MIT"],
      links: %{"GitHub" => @source_url},
      files: ~w(lib native .formatter.exs mix.exs README.md LICENSE)
    ]
  end

  defp docs do
    [
      main: "QuickJSEx",
      source_ref: "v#{@version}"
    ]
  end
end
