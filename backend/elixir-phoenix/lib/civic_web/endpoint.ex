# SPDX-License-Identifier: AGPL-3.0-or-later
# SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

defmodule CivicWeb.Endpoint do
  @moduledoc """
  Phoenix Endpoint for CivicConnect

  Handles HTTP requests and WebSocket connections.
  """

  use Phoenix.Endpoint, otp_app: :civic

  # Session configuration
  @session_options [
    store: :cookie,
    key: "_civic_key",
    signing_salt: "civic_signing",
    same_site: "Lax"
  ]

  socket "/live", Phoenix.LiveView.Socket,
    websocket: [connect_info: [session: @session_options]]

  socket "/socket", CivicWeb.UserSocket,
    websocket: true,
    longpoll: false

  # Serve static files
  plug Plug.Static,
    at: "/",
    from: :civic,
    gzip: false,
    only: CivicWeb.static_paths()

  # Code reloading in dev
  if code_reloading? do
    socket "/phoenix/live_reload/socket", Phoenix.LiveReloader.Socket
    plug Phoenix.LiveReloader
    plug Phoenix.CodeReloader
  end

  plug Phoenix.LiveDashboard.RequestLogger,
    param_key: "request_logger",
    cookie_key: "request_logger"

  plug Plug.RequestId
  plug Plug.Telemetry, event_prefix: [:phoenix, :endpoint]

  plug Plug.Parsers,
    parsers: [:urlencoded, :multipart, :json],
    pass: ["*/*"],
    json_decoder: Phoenix.json_library()

  plug Plug.MethodOverride
  plug Plug.Head
  plug Plug.Session, @session_options
  plug CivicWeb.Router
end
