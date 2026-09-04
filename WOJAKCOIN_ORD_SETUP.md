# Wojakcoin Ord (ordwoj) Setup

`ordwoj` is the Wojakcoin fork of [ordinals/ord](https://github.com/ordinals/ord), based on the same architecture as [ord-pepecoin](https://github.com/mvdnbrk/ord-pepecoin). It indexes **wojakinals** (inscriptions), **WJK-20** fungible token deploys/transfers, **WJK-721** collections (parent/delegate/properties), and **wojakmaps** (dogemap-style `{N}.wojakmap` block claims).

## Requirements

- Synced `wojakcoind` with `txindex=1` and RPC enabled
- Rust 1.89+ (to build from source)

Default Wojakcoin ports ([wojakcore](https://github.com/WojakCoinProj/wojakcore)):

| Service | Port |
|---------|------|
| P2P | 20759 |
| RPC | 20760 |

## Build

```bash
cd /root/ord-wojakcoin
cargo build --release
# binary: ./target/release/ordwoj
```

Chain parameters live in `/root/rust-wojakcoin-bitcoin` (patched `bitcoin` crate: magic `0x79a58d6f`, address prefix `W` / `0x49`, WIF `0xc9`).

## Configuration

Copy `ordwoj.yaml.example` to `~/.local/share/ordwoj/ordwoj.yaml` or pass `--config`:

```yaml
wojakcoin_rpc_username: "danny"
wojakcoin_rpc_password: "your_password"
rpc_url: "127.0.0.1:20760"
wojakcoin_data_dir: "/root/.wojakcoin"
data_dir: "/root/ord-wojakcoin-data"
index: "/root/ord-wojakcoin-data/index.redb"
first_inscription_height: 1
```

## Index and explorer

```bash
# One-time index build (runs until caught up)
ordwoj --config /path/to/ordwoj.yaml index update

# Block explorer + wallet API
ordwoj --config /path/to/ordwoj.yaml server --http-port 3080
```

Explorer URLs (after indexing):

- Home: `http://127.0.0.1:3080/`
- Inscription: `http://127.0.0.1:3080/inscription/<id>`
- Content: `http://127.0.0.1:3080/content/<id>`
- Address inscriptions: `http://127.0.0.1:3080/address/<Wk...>`

## Wallet / inscribe

```bash
ordwoj wallet create
ordwoj wallet receive
ordwoj wallet inscribe --fee-rate 10000 --file image.png
```

Use `--reinscribe` and WJK-721 tags per [docs/wjk-721.md](docs/wjk-721.md) for collections and wojakmaps provenance.

## Metaprotocols

| Name | Role |
|------|------|
| **wojakinals** | Wojakcoin ordinals / inscriptions (NFTs, images, JSON) |
| **WJK-20** | Fungible token inscriptions (`{"p":"wjk-20",...}` style); indexed as content, no built-in balance API |
| **WJK-721** | Extended envelope: parent, delegate, properties, compressed metadata |
| **wojakmaps** | Dogemap-style block claims: first `text/plain` body `{N}.wojakmap` wins that block |
| **WJK-721 collections** | Parent/child provenance via `parent` + `properties` tags |

## Nginx (public explorer)

Domains: **ord.wojakcoin.cash**, **ord.wojakcoin2017.xyz** → `127.0.0.1:3080`

```bash
sudo ./deploy/install-nginx.sh
sudo certbot --nginx -d ord.wojakcoin.cash -d ord.wojakcoin2017.xyz
```

Config: `deploy/nginx-ord-wojakcoin.conf`

## Related repos

- [WojakCoinProj/wojakcore](https://github.com/WojakCoinProj/wojakcore) — full node
- [reallyshadydev/rust-wojakcoin](https://github.com/reallyshadydev/rust-wojakcoin) — Rust chain types
- `/root/wojak-indexer` — UTXO/address indexer (separate from ord)
