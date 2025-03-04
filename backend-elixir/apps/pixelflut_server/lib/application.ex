defmodule PixelflutServer.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  def start(_type, _args) do
    # List all child processes to be supervised
    children = [
      # Starts a worker by calling: PixelflutServer.Worker.start_link(arg)
      # {PixelflutServer.Worker, arg},
      {Task.Supervisor, name: PixelflutServer.Net.SocketSupervisor},
      {PixelflutServer.Net.TcpServer, task_supervisor: PixelflutServer.Net.SocketSupervisor},
    ]

    # See https://hexdocs.pm/elixir/Supervisor.html
    # for other strategies and supported options
    opts = [strategy: :one_for_one, name: PixelflutServer.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
