# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

defmodule Civic.Repo do
  @moduledoc """
  Database repository for CivicConnect

  Uses PostgreSQL with PostGIS extension for spatial queries.
  """

  use Ecto.Repo,
    otp_app: :civic,
    adapter: Ecto.Adapters.Postgres
end
