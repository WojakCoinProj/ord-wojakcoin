mod ledger;
pub mod parse;
pub mod tables;

pub use ledger::Wjk20Updater;
pub use parse::{normalize_tick, ParsedWjk20, Wjk20Op};
pub use tables::{
  balance_key, clear_tables, open_read_tables, open_write_tables, DeployRecord, DomainRecord,
  EventRecord, PendingMint, PendingTransfer, Wjk20ReadTables, Wjk20WriteTables,
};

use super::*;

pub fn rebuild(index: &Index) -> Result {
  index.rebuild_wjk20()
}

pub fn rebuild_wojakmaps(index: &Index) -> Result {
  index.rebuild_wojakmap_claims()
}

pub fn rebuild_domains(index: &Index) -> Result {
  index.rebuild_domains()
}
