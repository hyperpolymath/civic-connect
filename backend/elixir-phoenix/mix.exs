# SPDX-License-Identifier: AGPL-3.0-or-later
# SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

defmodule Civic.MixProject do
  use Mix.Project

  def project do
    [
      app: :civic,
      version: "0.1.0",
      elixir: "~> 1.16",
      elixirc_paths: elixirc_paths(Mix.env()),
      start_permanent: Mix.env() == :prod,
      aliases: aliases(),
      deps: deps(),
      description: "CivicConnect real-time web layer - WebSocket, chat, Phoenix UI",
      package: package()
    ]
  end

  def application do
    [
      mod: {Civic.Application, []},
      extra_applications: [:logger, :runtime_tools]
    ]
  end

  defp elixirc_paths(:test), do: ["lib", "test/support"]
  defp elixirc_paths(_), do: ["lib"]

  defp deps do
    [
      # Phoenix Framework
      {:phoenix, "~> 1.7.10"},
      {:phoenix_html, "~> 4.0"},
      {:phoenix_live_reload, "~> 1.4", only: :dev},
      {:phoenix_live_view, "~> 0.20.1"},
      {:phoenix_live_dashboard, "~> 0.8.2"},

      # Database
      {:ecto_sql, "~> 3.11"},
      {:postgrex, ">= 0.0.0"},

      # HTTP Server
      {:bandit, "~> 1.2"},

      # Telemetry
      {:telemetry_metrics, "~> 0.6"},
      {:telemetry_poller, "~> 1.0"},

      # JSON
      {:jason, "~> 1.4"},

      # Assets
      {:esbuild, "~> 0.8", runtime: Mix.env() == :dev},
      {:tailwind, "~> 0.2.0", runtime: Mix.env() == :dev},

      # Security
      {:bcrypt_elixir, "~> 3.1"},

      # Background Jobs
      {:oban, "~> 2.17"},

      # Time handling
      {:timex, "~> 3.7"},

      # HTTP Client
      {:httpoison, "~> 2.2"},

      # Development & Testing
      {:floki, ">= 0.30.0", only: :test},
      {:ex_machina, "~> 2.7", only: :test},
      {:credo, "~> 1.7", only: [:dev, :test], runtime: false},
      {:dialyxir, "~> 1.4", only: [:dev, :test], runtime: false}
    ]
  end

  defp aliases do
    [
      setup: ["deps.get", "ecto.setup", "assets.setup", "assets.build"],
      "ecto.setup": ["ecto.create", "ecto.migrate", "run priv/repo/seeds.exs"],
      "ecto.reset": ["ecto.drop", "ecto.setup"],
      test: ["ecto.create --quiet", "ecto.migrate --quiet", "test"],
      "assets.setup": ["tailwind.install --if-missing", "esbuild.install --if-missing"],
      "assets.build": ["tailwind civic", "esbuild civic"],
      "assets.deploy": [
        "tailwind civic --minify",
        "esbuild civic --minify",
        "phx.digest"
      ]
    ]
  end

  defp package do
    [
      name: "civic",
      licenses: ["AGPL-3.0-or-later"],
      links: %{
        "GitHub" => "https://github.com/hyperpolymath/civic-connect"
      }
    ]
  end
end
