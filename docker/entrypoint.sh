#!/usr/bin/env bash
# First-run provisioning, then the terminal, then the relay.
#
# Every step is idempotent and writes into $WINEPREFIX, which is a volume. The
# first boot takes several minutes and downloads a terminal from MetaQuotes;
# later boots skip straight to starting it.
#
# The order is MetaQuotes' own, from their mt5linux.sh: report Windows 11,
# install the WebView2 runtime, then run the installer. It is not decoration —
# the installer draws a web UI, and without WebView2 it dies without a message.
set -uo pipefail

MODE="${1:-relay}"

WEBVIEW_URL="${WEBVIEW_URL:-https://msedge.sf.dl.delivery.mp.microsoft.com/filestreamingservice/files/f2910a1e-e5a6-4f17-b52d-7faf525d17f8/MicrosoftEdgeWebview2Setup.exe}"
MT5_SETUP_URL="${MT5_SETUP_URL:-https://download.mql5.com/cdn/web/metaquotes.software.corp/mt5/mt5setup.exe}"

MT5_DIR="${WINEPREFIX}/drive_c/Program Files/MetaTrader 5"
TERMINAL_UNIX="${MT5_DIR}/terminal64.exe"
STAMP_DIR="${WINEPREFIX}/.provisioned"

log() { printf '%s entrypoint %s\n' "$(date -Is)" "$*" >&2; }
die() { log "FATAL: $*"; exit 1; }
done_with() { [[ -f "${STAMP_DIR}/$1" ]]; }
mark_done() { touch "${STAMP_DIR}/$1"; }

[[ -n "${MT5_BRIDGE_TOKEN:-}" ]] || die "MT5_BRIDGE_TOKEN is required: this port reaches a live terminal"

# Skip Wine's Mono and Gecko prompts. Neither is needed: WebView2 provides the
# browser the installer wants, and nothing here runs .NET.
export WINEDLLOVERRIDES="${WINEDLLOVERRIDES:-mscoree,mshtml=}"

# The terminal is a GUI application and will not start without an X server,
# even when it is only ever driven over IPC.
if ! pgrep -x Xvfb >/dev/null; then
    # A RESTARTED CONTAINER KEEPS ITS FILESYSTEM, and the socket from the last
    # run outlives the process that made it. Xvfb then refuses to start —
    # "server is already active for display 99" — while the wait below sees the
    # stale socket and reports success. The terminal comes up with nowhere to
    # draw and exits half a second later with code 0 and no explanation, and
    # the relay answers every request with "File not found": the pipe belongs
    # to a terminal that is no longer there.
    #
    # Cost of this: the first start works, every restart afterwards does not.
    rm -f /tmp/.X11-unix/X99 /tmp/.X99-lock
    Xvfb :99 -screen 0 1024x768x16 -ac -nolisten tcp &
    for _ in $(seq 1 50); do
        pgrep -x Xvfb >/dev/null && [[ -e /tmp/.X11-unix/X99 ]] && break
        sleep 0.1
    done
    # Checked, not assumed: without an X server nothing below can work, and
    # every symptom of its absence points somewhere else.
    pgrep -x Xvfb >/dev/null || die "Xvfb did not start; the terminal has nowhere to draw"
fi
export DISPLAY=:99

mkdir -p "${STAMP_DIR}"

if [[ ! -d "${WINEPREFIX}/drive_c" ]]; then
    log "[1/4] initialising the Wine prefix ($(wine --version 2>&1 | head -1))"
    wineboot --init || die "wineboot failed"
    wineserver -w
fi

if ! done_with winver; then
    log "[2/4] reporting Windows 11 — the installer checks the version"
    winecfg -v=win11 || die "winecfg failed"
    wineserver -w
    mark_done winver
fi

if ! done_with webview2; then
    log "[3/4] installing the WebView2 runtime"
    curl -fsSL -o /tmp/webview2.exe "${WEBVIEW_URL}" || die "could not download WebView2"
    wine /tmp/webview2.exe /silent /install
    rm -f /tmp/webview2.exe
    # Deliberately no `wineserver -w` here: WebView2 leaves a service running,
    # so waiting for every Wine process to exit never returns.
    if ls "${WINEPREFIX}/drive_c/Program Files (x86)/Microsoft/EdgeWebView/Application" >/dev/null 2>&1; then
        mark_done webview2
    else
        die "WebView2 did not install; the MT5 installer will fail without it"
    fi
fi

if [[ ! -f "${TERMINAL_UNIX}" ]]; then
    log "[4/4] installing MetaTrader 5 (several minutes on first boot)"
    curl -fsSL -o /tmp/mt5setup.exe "${MT5_SETUP_URL}" || die "could not download the installer"
    # The installer detaches and its exit code is not meaningful under /auto.
    # Whether terminal64.exe exists afterwards is the only thing worth checking.
    wine /tmp/mt5setup.exe /auto >/dev/null 2>&1 &
    for _ in $(seq 1 "${MT5_INSTALL_WAIT:-60}"); do
        [[ -f "${TERMINAL_UNIX}" ]] && break
        sleep 10
    done
    rm -f /tmp/mt5setup.exe
    [[ -f "${TERMINAL_UNIX}" ]] || die "the terminal is still missing after the installer ran"
    log "terminal installed"
fi

# Without this the terminal refuses every order before it reaches the broker:
# `CLIENT_DISABLES_AT (10027)` while `Enabled=0`, and the same again while
# `Api=1`, which is the "disable automatic trading via external Python API"
# guard and applies to any external client, this one included. There is no
# button to press in a container, and neither a `/config:` start file nor
# xdotool reaches these — the terminal reads them from `common.ini` at startup
# and nowhere else.
#
# `Account=0` and `Profile=0` are the ones that make this stick. They are the
# "disable automated trading when the account has been changed" pair, and left
# at 1 the terminal switches trading off the moment anything logs in — which is
# every boot, because the platform logs in over IPC. Public images work around
# this by shipping a VNC so a human can press the toolbar button once; with
# these two at zero nobody has to.
#
# WRITTEN ONLY WHEN ABSENT. Editing a file the terminal owns is how this went
# wrong before: the round trip through UTF-16 left a second BOM in front of
# `[Experts]`, the section header stopped being one, and the terminal exited a
# second after start with no message worth the name. A prefix whose ini is
# already wrong is fixed by DELETING the file — the block below then writes a
# clean one.
if [[ "${MT5_ALGO_TRADING:-1}" == "1" && ! -f "${MT5_DIR}/config/common.ini" ]]; then
    mkdir -p "${MT5_DIR}/config"
    # UTF-16LE with a BOM and CRLF, like every other ini the terminal writes.
    {
        printf '\xff\xfe'
        printf '[Experts]\r\nEnabled=1\r\nApi=0\r\nAccount=0\r\nProfile=0\r\nAllowDllImport=0\r\n' \
            | iconv -f UTF-8 -t UTF-16LE
    } > "${MT5_DIR}/config/common.ini"
    log "wrote config/common.ini enabling algo trading (MT5_ALGO_TRADING=0 to skip)"
fi

# Build 6140 ships an assistant that opens two HTTP servers inside the terminal
# — MetaEditor on 22345, the terminal itself on 22346 — and enables them by
# default. Nothing here talks to them: this terminal is driven over its own
# pipe, and has neither an editor nor a human. Written on every boot rather
# than once, because the terminal puts the file back when a new build arrives.
#
# Same encoding as every other ini it reads: UTF-16LE, BOM, CRLF. Written any
# other way the section is not a section, the setting silently stays default,
# and the only sign is two ports still listening.
if [[ "${MT5_ASSISTANT:-0}" == "0" ]]; then
    mkdir -p "${MT5_DIR}/config"
    {
        printf '\xff\xfe'
        printf '[MCP.MetaEditor]\r\nEnable=0\r\n[MCP.MetaTrader]\r\nEnable=0\r\n' \
            | iconv -f UTF-8 -t UTF-16LE
    } > "${MT5_DIR}/config/assistant.ini"
    log "assistant servers off (MT5_ASSISTANT=1 to leave them on)"
fi

# NO `/config:` START FILE HERE, and that is a finding, not an oversight.
#
# MetaQuotes document a startup file with Login/Password/Server under
# `[Common]`, and it does work: the log says "successfully initialized from
# start config". But a terminal launched that way NEVER OPENS ITS IPC PIPE —
# the relay then answers every request with "File not found (os error 2)",
# which reads like a dead terminal while the terminal is plainly alive.
# Measured here, twice, on this image.
#
# So the terminal starts plainly and the platform logs it in over IPC. That
# changes the account under a running terminal, which is exactly what used to
# switch algo trading off — and what `Account=0` above now prevents.
STARTUP_INI="${MT5_DIR}/config/startup.ini"
rm -f "${STARTUP_INI}"

# THE LAUNCH COMMAND LIVES IN A FILE, AND ONLY THERE. A terminal behind the
# relay dies of its own accord now and then: the process disappears, the relay
# stays up, and every request comes back "no such pipe". Whatever restarts it
# should not be a person, and whatever restarts it must not carry a second copy
# of this command — so it calls this file instead:
#
#   docker exec -d <container> /opt/start-terminal.sh
cat > /opt/start-terminal.sh <<LAUNCH
#!/bin/sh
cd "${MT5_DIR}" || exit 1
pgrep -f terminal64.exe >/dev/null && exit 0
exec wine terminal64.exe ${MT5_PORTABLE:+/portable} >>/tmp/terminal.log 2>&1
LAUNCH
chmod +x /opt/start-terminal.sh

# ON A FRESH VOLUME THE INSTALLER STARTS A TERMINAL OF ITS OWN, and that
# instance shuts down a moment after the install. Checking for a running
# terminal once, or for a log file dated today, is fooled by it twice: the
# installer's instance is alive when the check runs, and it writes the log as
# it exits. The result was a healthy container with no terminal in it and a
# relay answering "File not found" to every request.
#
# So the terminal counts as up only after its process has stayed alive for
# MT5_ALIVE_SECONDS in a row, and it is started again whenever it is gone.
log "starting the terminal"
alive=0
for _ in $(seq 1 "${MT5_STARTUP_WAIT:-120}"); do
    if pgrep -f terminal64.exe >/dev/null; then
        alive=$((alive + 1))
        [[ ${alive} -ge ${MT5_ALIVE_SECONDS:-10} ]] && break
    else
        alive=0
        ( /opt/start-terminal.sh & )
    fi
    sleep 1
done
if [[ ${alive} -ge ${MT5_ALIVE_SECONDS:-10} ]]; then
    log "terminal is up"
else
    log "WARNING: the terminal did not stay up for ${MT5_ALIVE_SECONDS:-10}s"
fi


case "${MODE}" in
    relay)
        log "relay on ${MT5_RELAY_BIND:-0.0.0.0}:${MT5_RELAY_PORT}"
        export MT5_RELAY_BIND="${MT5_RELAY_BIND:-0.0.0.0}"
        exec wine /opt/mt5-relay.exe
        ;;
    terminal)
        log "terminal only, no relay"
        exec tail -f /dev/null
        ;;
    *) die "unknown mode '${MODE}' (relay|terminal)" ;;
esac
