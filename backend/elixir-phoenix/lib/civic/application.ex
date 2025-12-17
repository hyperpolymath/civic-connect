# SPDX-License-Identifier: AGPL-3.0-or-later
# SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

defmodule Civic.Application do
  @moduledoc """
  CivicConnect OTP Application

  Starts and supervises all application processes including:
  - Database connection pool
  - Phoenix endpoint
  - PubSub for real-time features
  - Presence tracking
  - Background job processing
  """

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      # Database repository
      Civic.Repo,
      # DNS cluster query
      {DNSCluster, query: Application.get_env(:civic, :dns_cluster_query) || :ignore},
      # PubSub for real-time
      {Phoenix.PubSub, name: Civic.PubSub},
      # Presence for tracking online users
      Civic.Presence,
      # Background jobs
      {Oban, Application.fetch_env!(:civic, Oban)},
      # Telemetry
      CivicWeb.Telemetry,
      # Phoenix endpoint (HTTP/WS server)
      CivicWeb.Endpoint
    ]

    opts = [strategy: :one_for_one, name: Civic.Supervisor]
    Supervisor.start_link(children, opts)
  end

  @impl true
  def config_change(changed, _new, removed) do
    CivicWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end
