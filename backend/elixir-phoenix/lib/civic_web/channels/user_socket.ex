# SPDX-License-Identifier: AGPL-3.0-or-later
# SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

defmodule CivicWeb.UserSocket do
  @moduledoc """
  WebSocket handler for CivicConnect

  Handles real-time connections for:
  - Chat messaging (E2E encrypted)
  - Event updates
  - Presence/typing indicators

  Security: All messages use E2E encryption (Signal protocol).
  Server only sees encrypted blobs.
  """

  use Phoenix.Socket

  ## Channels
  channel "chat:*", CivicWeb.ChatChannel
  channel "event:*", CivicWeb.EventChannel
  channel "user:*", CivicWeb.UserChannel

  @impl true
  def connect(params, socket, _connect_info) do
    # TODO: Verify JWT token from params
    # TODO: Load user from database
    # For now, require a token
    case params["token"] do
      nil ->
        :error

      _token ->
        # TODO: Verify token and get user_id
        {:ok, assign(socket, :user_id, nil)}
    end
  end

  @impl true
  def id(socket), do: "user_socket:#{socket.assigns.user_id}"
end
