# SPDX-License-Identifier: AGPL-3.0-or-later
# SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

defmodule Civic.Presence do
  @moduledoc """
  Presence tracking for CivicConnect

  Tracks which users are online and their current status.
  Used for:
  - "Who's online" indicators
  - Typing indicators in chat
  - Real-time event participant counts

  Privacy note: Presence only shows what users explicitly share.
  Location is never tracked via presence.
  """

  use Phoenix.Presence,
    otp_app: :civic,
    pubsub_server: Civic.PubSub
end
