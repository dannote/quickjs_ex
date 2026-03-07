defmodule QuickJSEx.MixProject do
  use Mix.Project

  @version "0.1.0"
  @source_url "https://github.com/dannote/quickjs_ex"

  def project do
    [
      app: :quickjs_ex,
      version: @version,
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      dialyzer: dialyzer(),
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
      {:credo, "~> 1.7", only: [:dev, :test], runtime: false},
      {:dialyxir, "~> 1.4", only: [:dev, :test], runtime: false},
      {:ex_dna, "~> 1.1", only: [:dev, :test], runtime: false},
      {:ex_doc, "~> 0.31", only: :dev, runtime: false}
    ]
  end

  defp dialyzer do
    [
      plt_add_apps: [:jason],
      plt_file: {:no_warn, "priv/plts/project.plt"}
    ]
  end

  defp package do
    [
      licenses: ["MIT"],
      links: %{"GitHub" => @source_url},
      files: ~w(
        lib
        native
        .formatter.exs
        mix.exs
        README.md
        LICENSE
        CHANGELOG.md
        Cargo.lock
        Cargo.toml
      )
    ]
  end

  defp docs do
    [
      main: "QuickJSEx",
      source_ref: "v#{@version}",
      extras: ["CHANGELOG.md"]
    ]
  end
end
