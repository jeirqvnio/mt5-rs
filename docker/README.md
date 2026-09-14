# MetaTrader 5 in Docker

MetaTrader 5 under Wine with `mt5-relay` in front of it. Works on amd64 and
arm64 hosts, including Apple Silicon.

## Start

```sh
echo "MT5_BRIDGE_TOKEN=$(openssl rand -hex 16)" > docker/.env
docker compose -f docker/compose.example.yaml up -d --build
```

The first start installs the terminal into the `/wine` volume. It takes a few
minutes and downloads the WebView2 runtime and the MetaTrader installer. Later
starts take seconds. The container reports healthy once the relay accepts
connections.

```sh
nc -z 127.0.0.1 18813
MT5_BRIDGE_TOKEN=... cargo run --example smoke
```

Log in from the client, not from the container:

```rust
let config = Config::relay("127.0.0.1:18813", &token).account(login, &password, "Broker-Demo");
let mt5 = Mt5::new(config);
mt5.connect().await?;
```

## Building from GitHub

The build context is the repository root. Pin a tag rather than a branch:

```yaml
services:
  mt5:
    image: mt5:relay
    build:
      context: https://github.com/jeirqvnio/mt5-rs.git#v0.1.0
      dockerfile: docker/Dockerfile
    command: ["relay"]
```

## Configuration

| variable | default | meaning |
|---|---|---|
| `MT5_BRIDGE_TOKEN` | required | shared secret the relay checks |
| `MT5_HOST_PORT` | `18813` | host port in `compose.example.yaml` |
| `MT5_IMAGE` | `mt5:relay` | image tag in `compose.example.yaml` |
| `MT5_RELAY_BIND` | `0.0.0.0` | listen address inside the container |
| `MT5_RELAY_PORT` | `18813` | listen port inside the container |
| `MT5_ALGO_TRADING` | `1` | write `common.ini` enabling algo trading on first start |
| `MT5_ASSISTANT` | `0` | leave the terminal's assistant HTTP servers on when `1` |
| `MT5_PORTABLE` | unset | start the terminal with `/portable` |
| `MT5_INSTALL_WAIT` | `60` | installer wait, in 10-second steps |
| `MT5_STARTUP_WAIT` | `120` | seconds allowed for the terminal to come up |
| `MT5_ALIVE_SECONDS` | `10` | seconds the terminal process must stay alive to count as up |

The container command is `relay` (terminal and relay) or `terminal` (terminal
only).

One container holds one terminal, and a terminal holds one account. For a
second account run a second service with its own port and volume.

## Notes

**Do not run the amd64 image on an ARM host.** Docker then runs Wine itself
under qemu, and Wine aborts at `wineboot --init`. The Dockerfile picks the
branch from `TARGETARCH`: native Wine on amd64, Hangover on arm64, where only
the terminal's x86-64 code is translated. Leave `platform:` unset.

**`SYS_PTRACE` and `seccomp:unconfined` are required.** Wine uses them for
exception handling. Without them the installer exits with code 29 and
installs nothing.

**Algo trading.** On first start the entrypoint writes `config/common.ini`
with `Enabled=1`, `Api=0`, `Account=0` and `Profile=0`. `Account=0` stops the
terminal from switching algo trading off when a client logs in. The file is
written only when absent. If the terminal exits right after start because the
file is malformed, delete it and restart the container.

**No `/config:` start file.** A terminal started with one does not open its
IPC pipe, so the relay cannot reach it.

**Restarting a terminal that died.** The relay stays up while the terminal
process can exit on its own, and every request then fails. Start it again
without restarting the container:

```sh
docker exec -d <container> /opt/start-terminal.sh
```

**Repairing the prefix.** Do not run winetricks in the arm64 prefix. On a
DLL load failure remove the volume and let the container provision it again.

**The port trades.** Publish it on `127.0.0.1` only.
