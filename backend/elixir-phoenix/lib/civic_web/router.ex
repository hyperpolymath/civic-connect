# SPDX-License-Identifier: AGPL-3.0-or-later
# SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

defmodule CivicWeb.Router do
  @moduledoc """
  Router for CivicConnect web application

  Routes are organized by:
  - Browser (HTML) routes
  - LiveView routes
  - API routes (proxied to Rust API for most operations)
  """

  use CivicWeb, :router

  pipeline :browser do
    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_live_flash
    plug :put_root_layout, html: {CivicWeb.Layouts, :root}
    plug :protect_from_forgery
    plug :put_secure_browser_headers
  end

  pipeline :api do
    plug :accepts, ["json"]
  end

  # Browser routes
  scope "/", CivicWeb do
    pipe_through :browser

    get "/", PageController, :home

    # Authentication
    get "/login", AuthController, :login
    get "/register", AuthController, :register
    get "/logout", AuthController, :logout

    # Events
    live "/events", EventLive.Index, :index
    live "/events/:id", EventLive.Show, :show
    live "/events/new", EventLive.New, :new

    # Chat
    live "/chat", ChatLive.Index, :index
    live "/chat/:room_id", ChatLive.Room, :show

    # User profile
    live "/profile", ProfileLive.Show, :show
    live "/profile/edit", ProfileLive.Edit, :edit

    # Dashboard (for logged-in users)
    live "/dashboard", DashboardLive, :index
  end

  # Development routes
  if Application.compile_env(:civic, :dev_routes) do
    import Phoenix.LiveDashboard.Router

    scope "/dev" do
      pipe_through :browser

      live_dashboard "/dashboard", metrics: CivicWeb.Telemetry
    end
  end
end
