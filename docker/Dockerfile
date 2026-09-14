# MetaTrader 5 under Wine, with mt5-relay exposing its IPC pipe over TCP.
#
# One container holds one terminal, and one terminal holds one account.
#
# The image carries Wine and the relay. The terminal is installed on first
# start into the /wine volume, so the image stays small and rebuilds do not
# re-download it. The prefix must stay on that volume: the pipe name is
# derived from the terminal's install path.
#
# Architecture is chosen from TARGETARCH, which BuildKit sets from the host:
#
#   amd64  native Wine from winehq.
#   arm64  Hangover: Wine built for ARM64, only the terminal's x86-64 code is
#          translated.
#
# Do not force the amd64 image onto an ARM host. Docker then runs Wine itself
# under qemu, and it aborts at `wineboot --init`.

ARG ARM_BASE=trixie-slim
ARG HANGOVER_VERSION=11.9
ARG HANGOVER_DIST=debian13_trixie

# --- relay: cross-compiled from this repository ------------------------------
FROM rust:1-bookworm AS relay
RUN apt-get update && \
    apt-get install -y --no-install-recommends gcc-mingw-w64-x86-64 && \
    rustup target add x86_64-pc-windows-gnu && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --features relay --bin mt5-relay \
        --target x86_64-pc-windows-gnu

# --- amd64: native Wine ------------------------------------------------------
FROM debian:bookworm-slim AS base-amd64
# Wine's recommends carry the graphics and font libraries the terminal needs.
RUN dpkg --add-architecture i386 && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates curl gnupg xvfb xauth procps tini && \
    mkdir -pm755 /etc/apt/keyrings && \
    curl -fsSL https://dl.winehq.org/wine-builds/winehq.key \
        -o /etc/apt/keyrings/winehq-archive.key && \
    curl -fsSL https://dl.winehq.org/wine-builds/debian/dists/bookworm/winehq-bookworm.sources \
        -o /etc/apt/sources.list.d/winehq-bookworm.sources && \
    apt-get update && \
    apt-get install --install-recommends -y winehq-staging && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# --- arm64: Hangover ---------------------------------------------------------
FROM debian:${ARM_BASE} AS base-arm64
ARG HANGOVER_VERSION
ARG HANGOVER_DIST
# Hangover ships a tar of .deb files per distribution, not a repository.
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates curl xvfb xauth procps tini \
        libgl1 libglx-mesa0 libfreetype6 libfontconfig1 libxrender1 \
        libxcursor1 libxi6 libxrandr2 libxinerama1 libxcomposite1 && \
    curl -fsSL -o /tmp/hangover.tar \
        "https://github.com/AndreRH/hangover/releases/download/hangover-${HANGOVER_VERSION}/hangover_${HANGOVER_VERSION}_${HANGOVER_DIST}_arm64.tar" && \
    mkdir -p /tmp/hangover && \
    tar -xf /tmp/hangover.tar -C /tmp/hangover && \
    apt-get install -y --no-install-recommends $(find /tmp/hangover -name '*.deb') && \
    apt-get clean && \
    rm -rf /tmp/hangover /tmp/hangover.tar /var/lib/apt/lists/*

# --- runtime -----------------------------------------------------------------
FROM base-${TARGETARCH} AS runtime

ENV DEBIAN_FRONTEND=noninteractive \
    WINEPREFIX=/wine \
    WINEARCH=win64 \
    WINEDEBUG=-all \
    DISPLAY=:99 \
    MT5_PATH="C:\\Program Files\\MetaTrader 5\\terminal64.exe" \
    MT5_RELAY_BIND=0.0.0.0 \
    MT5_RELAY_PORT=18813

COPY --from=relay /src/target/x86_64-pc-windows-gnu/release/mt5-relay.exe /opt/mt5-relay.exe
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

EXPOSE 18813

# The relay binds only after the terminal is up, so a refused connection means
# provisioning is not finished. The start period covers the first-run install.
HEALTHCHECK --interval=30s --timeout=5s --start-period=900s --retries=3 \
    CMD bash -c "</dev/tcp/127.0.0.1/${MT5_RELAY_PORT}" || exit 1

# tini reaps Wine's children. Without it restarted terminals leave zombies.
# `-s` registers it as a subreaper, so it still reaps when compose `init: true`
# puts docker-init in front of it as PID 1.
ENTRYPOINT ["/usr/bin/tini", "-s", "--", "/usr/local/bin/entrypoint.sh"]
CMD ["relay"]
