# ordwoj

[![CI](https://github.com/mvdnbrk/ord-wojakcoin/actions/workflows/ci.yaml/badge.svg)](https://github.com/mvdnbrk/ord-wojakcoin/actions/workflows/ci.yaml)
[![Release](https://img.shields.io/github/v/release/mvdnbrk/ord-wojakcoin)](https://github.com/mvdnbrk/ord-wojakcoin/releases/latest)

Ordinal indexer and block explorer for **Wojakcoin**. Originally forked from [apezord/ord-dogecoin](https://github.com/apezord/ord-dogecoin) (based on [ordinals/ord](https://github.com/ordinals/ord) v0.5.1), but extensively rewritten with modernized dependencies (redb 3.x, axum 0.8, reqwest 0.12), a standalone wallet with local key management, batch inscription support, and the [WJK-721](docs/wjk-721.md) extended inscription envelope specification.

The indexer and explorer support all inscription content types — **wojakinals** (NFTs/media), **wojakmaps** (dogemap-style `{N}.wojakmap` block claims), **`.wjk` domains** (dash-style `name.wjk` registrations), **WJK-721** collections (parent/delegate/properties), and **WJK-20** deploy/transfer JSON with an indexed balance ledger and HTTP APIs.

## Requirements

- Synced `wojakcoind` node with `-txindex`

## Installation

### Pre-built binary

```bash
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/mvdnbrk/ord-wojakcoin/main/install.sh | sh
```

This auto-detects your platform and installs the latest release to `/usr/local/bin`. See [all releases](https://github.com/mvdnbrk/ord-wojakcoin/releases).

### Build from source

Requires Rust 1.89+:

```bash
git clone https://github.com/mvdnbrk/ord-wojakcoin.git
cd ord-wojakcoin
cargo build --release
```

The binary is at `./target/release/ordwoj`.

## Configuration

`ordwoj` can be configured with command-line flags, a YAML configuration file, or both. Command-line flags take precedence over the configuration file.

### Configuration file

Create an `ordwoj.yaml` file:

```yaml
wojakcoin_rpc_username: "your_rpc_user"
wojakcoin_rpc_password: "your_rpc_password"
rpc_url: "127.0.0.1:20760"
data_dir: "/data/ordwoj"
index: "/data/ordwoj/index.redb"
```

The configuration file is loaded from the first location found:

1. `--config <path>` — explicit path (errors if not found)
2. `--config-dir <dir>/ordwoj.yaml`
3. `--data-dir <dir>/ordwoj.yaml`
4. Default data directory (`ordwoj.yaml`)

All configuration file fields are optional:

| Field | Description |
|---|---|
| `wojakcoin_rpc_username` | RPC username (alternative to cookie auth) |
| `wojakcoin_rpc_password` | RPC password (alternative to cookie auth) |
| `rpc_url` | Wojakcoin Core RPC URL |
| `wojakcoin_data_dir` | Wojakcoin Core data directory |
| `data_dir` | ordwoj data directory |
| `index` | Path to the index database |
| `index_sats` | Track location of all satoshis (`true`/`false`) |
| `cookie_file` | Path to RPC cookie file |
| `server_url` | URL of the ordwoj server |
| `hidden` | List of inscription IDs to hide |

### Authentication

RPC authentication is resolved in this order:

1. `wojakcoin_rpc_username` + `wojakcoin_rpc_password` in config file (username/password auth)
2. `--cookie-file` flag or `cookie_file` in config (cookie auth)
3. Default cookie file location (`~/.wojakcoin/.cookie`)

## Usage

### With a configuration file

```bash
ordwoj --config /path/to/ordwoj.yaml server --http-port 3080
ordwoj --config /path/to/ordwoj.yaml index update
```

### With command-line flags

```bash
ordwoj --rpc-url 127.0.0.1:20760 --cookie-file ~/.wojakcoin/.cookie server --http-port 3080
ordwoj --rpc-url 127.0.0.1:20760 --cookie-file ~/.wojakcoin/.cookie index update
```

### Export inscriptions to TSV

```bash
ordwoj index export --include-addresses > inscriptions.tsv
```

### Compact the database

```bash
ordwoj index compact
```

### Rebuild WJK-20 ledger

If you upgraded from a build without WJK-20 tables, or need to refresh balances:

```bash
ordwoj index rebuild-wjk20
```

### JSON API

The server returns JSON when the `Accept: application/json` header is set:

#### WJK-20 and wojakmaps

| Endpoint | Description |
|---|---|
| `GET /api/tokens` | All deployed WJK-20 tokens |
| `GET /api/tokens/{tick}` | One token (`max`, `lim`, deploy inscription, etc.) |
| `GET /api/deploys?limit=&offset=` | Deploy events |
| `GET /api/balances/{address}` | All WJK-20 balances for an address |
| `GET /api/balances/{address}/{tick}` | Balance for one tick |
| `GET /api/wojakmaps?limit=&offset=` | Wojakmap block claims (`{N}.wojakmap`, first wins) |
| `GET /api/wojakmap/{block}` | Claim for a block number |
| `GET /api/wojakmaps/{inscription_id}` | Claim owned by an inscription id |
| `GET /api/collections?limit=&offset=` | WJK-721 collection roots (inscriptions with children) |
| `GET /api/collections/{inscription_id}` | Collection detail + child inscription IDs |
| `GET /api/domains?limit=&offset=` | Registered `.wjk` domains (newest first) |
| `GET /api/domains/stats` | Domain total + unique owners |
| `GET /api/domains/name/{name}` | Availability lookup (strips `.wjk`) |
| `GET /api/domains/address/{address}` | Domains owned by address |

WJK-20 JSON inscriptions use `{"p":"wjk-20","op":"deploy|mint|transfer",...}` (BRC-20 style). Balances update on mint and when a transfer inscription is sent to a recipient.

#### Explorer (HTML or JSON with `Accept: application/json`)

The server returns JSON when the `Accept: application/json` header is set:

```bash
curl -s -H "Accept: application/json" http://localhost:3080/status
curl -s -H "Accept: application/json" http://localhost:3080/inscription/<inscription_id>
curl -s -H "Accept: application/json" http://localhost:3080/inscriptions
curl -s -H "Accept: application/json" http://localhost:3080/output/<outpoint>
curl -s -H "Accept: application/json" http://localhost:3080/block/<height>
curl -s -H "Accept: application/json" http://localhost:3080/address/<address>
curl -s -H "Accept: application/json" http://localhost:3080/children/<inscription_id>
curl -s -H "Accept: application/json" http://localhost:3080/children/<inscription_id>/<page>
curl -s -H "Accept: application/json" http://localhost:3080/parents/<inscription_id>
curl -s -H "Accept: application/json" http://localhost:3080/parents/<inscription_id>/<page>
curl -s http://localhost:3080/blockcount
curl -s http://localhost:3080/content/<inscription_id>
```

The `/address/<address>` endpoint returns inscription IDs and their corresponding output points, useful for inscription-aware UTXO selection in wallets:

```json
{
  "inscriptions": ["<inscription_id>", ...],
  "outputs": ["<txid>:<vout>", ...]
}
```

The `/status` endpoint returns index information:

```json
{
  "address_index": true,
  "chain": "mainnet",
  "height": 945000,
  "inscriptions": 12345,
  "sat_index": false,
  "unrecoverably_reorged": false
}
```

Raw inscription content is always available at `/content/<inscription_id>`.

## Wallet

`ordwoj` includes a standalone wallet with local key management. Keys are derived locally (BIP-44 `m/44'/3434'/0'`) and stored in `wallet.redb` with restricted permissions (0600).

Signing and coin selection are performed locally, ensuring inscriptions are protected from accidental spending.

### Commands

| Command | Description |
|---|---|
| `create` | Create a new wallet and display the mnemonic |
| `receive` | Generate a new receive address |
| `balance` | Display the wallet's balance |
| `send` | Send a specific amount, inscription, or satpoint |
| `inscribe` | Create a new inscription |
| `inscriptions` | List all inscriptions held by the wallet |
| `addresses` | List all addresses in the wallet |
| `restore` | Restore a wallet from a mnemonic |

### Example Usage

```bash
# Create a new wallet
ordwoj wallet create

# Generate a receive address
ordwoj wallet receive

# Check balance
ordwoj wallet balance

# Send an inscription
ordwoj wallet send <DESTINATION_ADDRESS> <INSCRIPTION_ID>

# Send 100 pep (requires unit: pep or rib)
ordwoj wallet send <DESTINATION_ADDRESS> 100pep
```

### Inscribing

```bash
ordwoj wallet inscribe --file /path/to/file.png --title "Optional Title"
ordwoj wallet inscribe --dry-run --file /path/to/file.png
```

Inscriptions use P2SH `script_sig` transactions (Wojakcoin has no SegWit). Large files are split across multiple chained transactions using 240-byte data chunks. Reveal transactions are signed locally.

### Batch Inscribing

Inscribe multiple files in a single operation:

```bash
ordwoj wallet inscribe --batch batch.yaml
```

Example `batch.yaml`:

```yaml
# Optional parents for all inscriptions in this batch
parents:
  - "0000000000000000000000000000000000000000000000000000000000000000i0"

inscriptions:
    # path to inscription content
  - file: first.png
    # title (optional)
    title: "Optional Title"
    # destination (optional, if no destination is specified a new wallet change address will be used)
    destination: PXvn95h8m6x4oGorNVerA2F4FFRpqMqwAM

  - file: second.png

  # Inscription to delegate content to
  - delegate: "1111111111111111111111111111111111111111111111111111111111111111i0"
```

## Credits

- [ordinals/ord](https://github.com/ordinals/ord) — Original Bitcoin ordinals indexer
- [apezord/ord-dogecoin](https://github.com/apezord/ord-dogecoin) — Dogecoin adaptation with `script_sig` support

## License

[CC0-1.0](LICENSE)
