#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SITE_NAME="ord-wojakcoin"
SRC="$ROOT/deploy/nginx-ord-wojakcoin.conf"
DEST="/etc/nginx/sites-available/$SITE_NAME"

if [[ "$(id -u)" -ne 0 ]]; then
  echo "Run as root: sudo $0"
  exit 1
fi

mkdir -p /var/www/certbot
cp "$SRC" "$DEST"
ln -sf "$DEST" "/etc/nginx/sites-enabled/$SITE_NAME"
nginx -t
systemctl reload nginx

echo "Installed $SITE_NAME"
echo "  ord.wojakcoin.cash -> http://127.0.0.1:3080"
echo "  ord.wojakcoin2017.xyz -> http://127.0.0.1:3080"
echo ""
echo "After ordwoj is running, issue TLS:"
echo "  certbot --nginx -d ord.wojakcoin.cash -d ord.wojakcoin2017.xyz"
