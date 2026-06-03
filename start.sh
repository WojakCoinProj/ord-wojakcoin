#!/usr/bin/env bash
# Start ordwoj (Wojakcoin ord indexer + explorer).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ORDWOJ="${ORDWOJ_BIN:-$ROOT/target/release/ordwoj}"
DATA_DIR="${ORDWOJ_DATA_DIR:-/root/ord-wojakcoin-data}"
HTTP_PORT="${ORDWOJ_HTTP_PORT:-3080}"
WOJAK_CONF="${WOJAKCOIN_CONF:-/root/.wojakcoin/wojakcoin.conf}"
PID_DIR="$DATA_DIR"
INDEX_PID="$PID_DIR/ordwoj-index.pid"
SERVER_PID="$PID_DIR/ordwoj-server.pid"
LOG_DIR="$DATA_DIR/logs"

usage() {
  cat <<'EOF'
Usage: ./start.sh <command>

Commands:
  index     Run index update only (foreground; no explorer)
  server    Explorer + API; indexes in a background thread (default 3080)
  all       Same as server (index + explorer in one process)
  stop      Stop background server (or index-only job)
  status    Show running ordwoj process

Environment:
  ORDWOJ_BIN, ORDWOJ_DATA_DIR, ORDWOJ_HTTP_PORT, WOJAKCOIN_CONF
  Reads rpcuser/rpcpassword/rpcport from WOJAKCOIN_CONF when set.
EOF
}

load_wojak_rpc() {
  if [[ ! -f "$WOJAK_CONF" ]]; then
    echo "Missing wojakcoin.conf: $WOJAK_CONF" >&2
    exit 1
  fi
  local line key val
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    [[ "$line" != *=* ]] && continue
    key="${line%%=*}"
    val="${line#*=}"
    case "$key" in
      rpcuser) export ORDWOJ_WOJAKCOIN_RPC_USERNAME="$val" ;;
      rpcpassword) export ORDWOJ_WOJAKCOIN_RPC_PASSWORD="$val" ;;
      rpcport) export _WOJAK_RPC_PORT="$val" ;;
    esac
  done < "$WOJAK_CONF"
  export ORDWOJ_RPC_URL="${ORDWOJ_RPC_URL:-127.0.0.1:${_WOJAK_RPC_PORT:-20760}}"
  export ORDWOJ_WOJAKCOIN_DATA_DIR="${ORDWOJ_WOJAKCOIN_DATA_DIR:-/root/.wojakcoin}"
}

export_ordwoj_env() {
  load_wojak_rpc
  export ORDWOJ_DATA_DIR="$DATA_DIR"
  export ORDWOJ_INDEX="$DATA_DIR/index.redb"
  mkdir -p "$DATA_DIR" "$LOG_DIR"
}

require_binary() {
  if [[ ! -x "$ORDWOJ" ]]; then
    echo "ordwoj not found at $ORDWOJ — run: cd $ROOT && cargo build --release" >&2
    exit 1
  fi
}

pid_running() {
  local pidfile="$1"
  [[ -f "$pidfile" ]] || return 1
  local pid
  pid="$(cat "$pidfile")"
  kill -0 "$pid" 2>/dev/null
}

cmd_index() {
  require_binary
  export_ordwoj_env
  exec "$ORDWOJ" index update
}

cmd_server() {
  require_binary
  export_ordwoj_env
  exec "$ORDWOJ" server --http-port "$HTTP_PORT" --address 127.0.0.1
}

cmd_all() {
  # Server opens the index and updates it internally — do not run index update separately.
  cmd_server_bg
}

cmd_server_bg() {
  require_binary
  export_ordwoj_env
  if pid_running "$SERVER_PID"; then
    echo "Server already running (pid $(cat "$SERVER_PID"))"
  else
    echo "Starting ordwoj (index + explorer) on http://127.0.0.1:$HTTP_PORT"
    echo "Log: $LOG_DIR/server.log"
    nohup "$ORDWOJ" server --http-port "$HTTP_PORT" --address 127.0.0.1 \
      >>"$LOG_DIR/server.log" 2>&1 &
    echo $! >"$SERVER_PID"
  fi
  echo "Explorer: http://127.0.0.1:$HTTP_PORT/"
}

cmd_stop() {
  local f
  for f in "$INDEX_PID" "$SERVER_PID"; do
    if pid_running "$f"; then
      kill "$(cat "$f")" 2>/dev/null || true
      rm -f "$f"
      echo "Stopped $(basename "$f")"
    fi
  done
  pkill -f "$ORDWOJ.*index update" 2>/dev/null || true
}

cmd_status() {
  status_one index "$INDEX_PID"
  status_one server "$SERVER_PID"
}

status_one() {
  local label="$1" pidfile="$2"
  if pid_running "$pidfile"; then
    echo "$label: running (pid $(cat "$pidfile"))"
  else
    echo "$label: stopped"
    rm -f "$pidfile"
  fi
}

main() {
  local cmd="${1:-}"
  case "$cmd" in
    index) cmd_index ;;
    server) cmd_server_bg ;;
    all) cmd_all ;;
    stop) cmd_stop ;;
    status) cmd_status ;;
    -h|--help|help|"") usage ;;
    *)
      echo "Unknown command: $cmd" >&2
      usage >&2
      exit 1
      ;;
  esac
}

main "$@"
