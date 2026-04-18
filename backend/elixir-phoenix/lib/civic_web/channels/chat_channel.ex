# SPDX-License-Identifier: AGPL-3.0-or-later
# SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

defmodule CivicWeb.ChatChannel do
  @moduledoc """
  Real-time chat channel

  Handles WebSocket messages for encrypted chat.
  All message content is E2E encrypted - server only routes encrypted blobs.

  Privacy guarantees:
  - Message content encrypted with Signal protocol
  - Server cannot read message content
  - Metadata minimized (no read receipts by default)
  """

  use CivicWeb, :channel

  alias Civic.Presence

  @impl true
  def join("chat:" <> room_id, _params, socket) do
    # TODO: Verify user has access to this room
    send(self(), :after_join)
    {:ok, assign(socket, :room_id, room_id)}
  end

  @impl true
  def handle_info(:after_join, socket) do
    # Track presence
    {:ok, _} =
      Presence.track(socket, socket.assigns.user_id, %{
        online_at: inspect(System.system_time(:second))
      })

    # Send current presence list
    push(socket, "presence_state", Presence.list(socket))

    {:noreply, socket}
  end

  @impl true
  def handle_in("message:send", %{"encrypted" => encrypted_blob}, socket) do
    # Broadcast encrypted message to room
    # Server never decrypts - just routes the blob
    broadcast!(socket, "message:new", %{
      "sender_id" => socket.assigns.user_id,
      "encrypted" => encrypted_blob,
      "sent_at" => DateTime.utc_now() |> DateTime.to_iso8601()
    })

    # TODO: Store in database for offline users
    # TODO: Queue for delivery to offline recipients

    {:reply, :ok, socket}
  end

  @impl true
  def handle_in("typing:start", _params, socket) do
    broadcast_from!(socket, "typing:start", %{
      "user_id" => socket.assigns.user_id
    })

    {:noreply, socket}
  end

  @impl true
  def handle_in("typing:stop", _params, socket) do
    broadcast_from!(socket, "typing:stop", %{
      "user_id" => socket.assigns.user_id
    })

    {:noreply, socket}
  end
end
