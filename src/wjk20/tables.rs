use {
  super::*,
  crate::index::{
    entry::{InscriptionIdValue, WojakmapClaimEntryValue},
    WJK20_BALANCE, WJK20_DEPLOY, WJK20_EVENT_BY_NUMBER, WJK20_PENDING_MINT,
    WJK20_PENDING_TRANSFER, WJK20_TICK_TO_NUMBERS, WJKMAP_BLOCK_TO_CLAIM, WJKMAP_ROOT_NUMBER,
    WJK_DOMAIN_ADDRESS_TO_NAMES, WJK_DOMAIN_ID_TO_NAME, WJK_DOMAIN_NAME,
  },
  redb::{
    MultimapTable, ReadOnlyMultimapTable, ReadOnlyTable, ReadTransaction, ReadableMultimapTable,
    ReadableTable, Table, WriteTransaction,
  },
};

pub fn balance_key(tick: &str, address: &str) -> Vec<u8> {
  let tick = parse::normalize_tick(tick);
  format!("{tick}\0{address}").into_bytes()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployRecord {
  pub tick: String,
  pub max: String,
  pub lim: String,
  pub dec: u8,
  pub inscription_id: String,
  pub inscription_number: u32,
  pub height: u32,
  pub deployer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
  pub op: String,
  pub tick: String,
  pub amt: Option<String>,
  pub max: Option<String>,
  pub lim: Option<String>,
  pub inscription_id: String,
  pub inscription_number: u32,
  pub height: u32,
  pub address: Option<String>,
  pub to_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTransfer {
  pub tick: String,
  pub amt: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingMint {
  pub tick: String,
  pub amt: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRecord {
  pub name: String,
  pub full_name: String,
  pub inscription_id: String,
  pub inscription_number: u32,
  pub owner_address: String,
  pub height: u32,
  pub timestamp: u32,
}

pub struct Wjk20WriteTables<'wtx> {
  pub deploy: Table<'wtx, &'static str, &'static [u8]>,
  pub balance: Table<'wtx, &'static [u8], &'static [u8]>,
  pub pending_transfer: Table<'wtx, &'static InscriptionIdValue, &'static [u8]>,
  pub pending_mint: Table<'wtx, &'static InscriptionIdValue, &'static [u8]>,
  pub event_by_number: Table<'wtx, u32, &'static [u8]>,
  pub tick_to_numbers: MultimapTable<'wtx, &'static str, u32>,
  pub wojakmap_roots: Table<'wtx, u32, u8>,
  pub wojakmap_claims: Table<'wtx, u32, WojakmapClaimEntryValue>,
  pub domain_name: Table<'wtx, &'static str, &'static [u8]>,
  pub domain_id_to_name: Table<'wtx, &'static InscriptionIdValue, &'static str>,
  pub domain_address_to_names: MultimapTable<'wtx, &'static str, &'static str>,
}

pub struct Wjk20ReadTables {
  pub deploy: ReadOnlyTable<&'static str, &'static [u8]>,
  pub balance: ReadOnlyTable<&'static [u8], &'static [u8]>,
  pub pending_transfer: ReadOnlyTable<&'static InscriptionIdValue, &'static [u8]>,
  pub pending_mint: ReadOnlyTable<&'static InscriptionIdValue, &'static [u8]>,
  pub event_by_number: ReadOnlyTable<u32, &'static [u8]>,
  pub tick_to_numbers: ReadOnlyMultimapTable<&'static str, u32>,
  pub wojakmap_roots: ReadOnlyTable<u32, u8>,
  pub wojakmap_claims: ReadOnlyTable<u32, WojakmapClaimEntryValue>,
  pub domain_name: ReadOnlyTable<&'static str, &'static [u8]>,
  pub domain_id_to_name: ReadOnlyTable<&'static InscriptionIdValue, &'static str>,
  pub domain_address_to_names: ReadOnlyMultimapTable<&'static str, &'static str>,
}

pub fn open_write_tables<'wtx>(wtx: &'wtx WriteTransaction) -> Result<Wjk20WriteTables<'wtx>> {
  Ok(Wjk20WriteTables {
    deploy: wtx.open_table(WJK20_DEPLOY)?,
    balance: wtx.open_table(WJK20_BALANCE)?,
    pending_transfer: wtx.open_table(WJK20_PENDING_TRANSFER)?,
    pending_mint: wtx.open_table(WJK20_PENDING_MINT)?,
    event_by_number: wtx.open_table(WJK20_EVENT_BY_NUMBER)?,
    tick_to_numbers: wtx.open_multimap_table(WJK20_TICK_TO_NUMBERS)?,
    wojakmap_roots: wtx.open_table(WJKMAP_ROOT_NUMBER)?,
    wojakmap_claims: wtx.open_table(WJKMAP_BLOCK_TO_CLAIM)?,
    domain_name: wtx.open_table(WJK_DOMAIN_NAME)?,
    domain_id_to_name: wtx.open_table(WJK_DOMAIN_ID_TO_NAME)?,
    domain_address_to_names: wtx.open_multimap_table(WJK_DOMAIN_ADDRESS_TO_NAMES)?,
  })
}

pub fn open_read_tables(rtx: &ReadTransaction) -> Result<Wjk20ReadTables> {
  Ok(Wjk20ReadTables {
    deploy: rtx.open_table(WJK20_DEPLOY)?,
    balance: rtx.open_table(WJK20_BALANCE)?,
    pending_transfer: rtx.open_table(WJK20_PENDING_TRANSFER)?,
    pending_mint: rtx.open_table(WJK20_PENDING_MINT)?,
    event_by_number: rtx.open_table(WJK20_EVENT_BY_NUMBER)?,
    tick_to_numbers: rtx.open_multimap_table(WJK20_TICK_TO_NUMBERS)?,
    wojakmap_roots: rtx.open_table(WJKMAP_ROOT_NUMBER)?,
    wojakmap_claims: rtx.open_table(WJKMAP_BLOCK_TO_CLAIM)?,
    domain_name: rtx.open_table(WJK_DOMAIN_NAME)?,
    domain_id_to_name: rtx.open_table(WJK_DOMAIN_ID_TO_NAME)?,
    domain_address_to_names: rtx.open_multimap_table(WJK_DOMAIN_ADDRESS_TO_NAMES)?,
  })
}

pub fn clear_tables(tables: &mut Wjk20WriteTables<'_>) -> Result {
  clear_wjk20_ledger(tables)?;
  tables.wojakmap_roots.retain(|_, _| false)?;
  clear_wojakmap_claims(tables)?;
  clear_domains(tables)?;
  Ok(())
}

/// Clear fungible-token ledger only (keeps wojakmap claims / domains intact).
pub fn clear_wjk20_ledger(tables: &mut Wjk20WriteTables<'_>) -> Result {
  tables.deploy.retain(|_, _| false)?;
  tables.balance.retain(|_, _| false)?;
  tables.pending_transfer.retain(|_, _| false)?;
  tables.pending_mint.retain(|_, _| false)?;
  tables.event_by_number.retain(|_, _| false)?;

  use redb::ReadableMultimapTable;
  let mut entries = Vec::new();
  for entry in tables.tick_to_numbers.iter()? {
    let (tick, numbers) = entry?;
    let tick = tick.value().to_string();
    for number in numbers {
      entries.push((tick.clone(), number?.value()));
    }
  }
  for (tick, number) in entries {
    tables.tick_to_numbers.remove(tick.as_str(), &number)?;
  }

  Ok(())
}

pub fn clear_wojakmap_claims(tables: &mut Wjk20WriteTables<'_>) -> Result {
  tables.wojakmap_claims.retain(|_, _| false)?;
  Ok(())
}

pub fn clear_domains(tables: &mut Wjk20WriteTables<'_>) -> Result {
  tables.domain_name.retain(|_, _| false)?;
  tables.domain_id_to_name.retain(|_, _| false)?;

  use redb::ReadableMultimapTable;
  let mut entries = Vec::new();
  for entry in tables.domain_address_to_names.iter()? {
    let (address, names) = entry?;
    let address = address.value().to_string();
    for name in names {
      entries.push((address.clone(), name?.value().to_string()));
    }
  }
  for (address, name) in entries {
    tables
      .domain_address_to_names
      .remove(address.as_str(), name.as_str())?;
  }
  Ok(())
}

pub fn migrate_tx(tx: &WriteTransaction) -> Result {
  tx.open_table(WJK20_DEPLOY)?;
  tx.open_table(WJK20_BALANCE)?;
  tx.open_table(WJK20_PENDING_TRANSFER)?;
  tx.open_table(WJK20_PENDING_MINT)?;
  tx.open_table(WJK20_EVENT_BY_NUMBER)?;
  tx.open_multimap_table(WJK20_TICK_TO_NUMBERS)?;
  tx.open_table(WJKMAP_ROOT_NUMBER)?;
  tx.open_table(WJKMAP_BLOCK_TO_CLAIM)?;
  tx.open_table(WJK_DOMAIN_NAME)?;
  tx.open_table(WJK_DOMAIN_ID_TO_NAME)?;
  tx.open_multimap_table(WJK_DOMAIN_ADDRESS_TO_NAMES)?;
  Ok(())
}

pub fn amount_to_string(amt: u128) -> String {
  amt.to_string()
}

pub fn parse_balance(bytes: &[u8]) -> Result<u128> {
  let s = std::str::from_utf8(bytes)?;
  Ok(s.parse()?)
}
