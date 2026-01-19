#!/usr/bin/env bash
set -e

APP_DIR="$HOME/.melodybrain"
BIN="$APP_DIR/melodybrain"
PID="$APP_DIR/melodybrain.pid"
VERSION="v0.1.0"
REPO="https://github.com/nonnorm/melodybrain/releases/download/$VERSION"

mkdir -p "$APP_DIR"

OS="$(uname | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    armv7l) ARCH="armv7" ;;
    armv6l) ARCH="armv6" ;;
    riscv64) ARCH="riscv64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

BINARY_URL="$REPO/melodybrain-$OS-$ARCH.tar.gz"

case "$1" in
    install)
        echo "Installing MelodyBrain..."
        curl -L "$BINARY_URL" -o /tmp/melodybrain.tar.gz
        tar -xzf /tmp/melodybrain.tar.gz -C "$APP_DIR"
        chmod +x "$BIN"
        echo "Installed to $BIN"
        echo "Run '$0 start' to launch MelodyBrain in the background."
        ;;

    start)
        if [ -f "$PID" ] && kill -0 $(cat "$PID") 2>/dev/null; then
            echo "MelodyBrain already running (PID $(cat "$PID"))"
            exit 1
        fi
        echo "Starting MelodyBrain in background..."
        nohup "$BIN" > /dev/null 2>&1 &
        echo $! > "$PID"
        echo "MelodyBrain started on port 33445!"
        ;;

    stop)
        if [ -f "$PID" ]; then
            echo "Stopping MelodyBrain..."
            kill $(cat "$PID") && rm "$PID"
            echo "Stopped."
        else
            echo "MelodyBrain not running."
        fi
        ;;

    uninstall)
        echo "Stopping (if running) and uninstalling MelodyBrain..."
        "$0" stop || true
        rm -rf "$APP_DIR"
        echo "MelodyBrain removed."
        ;;

    status)
        if [ -f "$PID" ] && kill -0 $(cat "$PID") 2>/dev/null; then
            echo "MelodyBrain is running (PID $(cat "$PID"))"
        else
            echo "MelodyBrain is not running."
        fi
        ;;

    *)
        echo "Usage: $0 {install|start|stop|status|uninstall}"
        exit 1
        ;;
esac

