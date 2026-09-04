use super::{tables::*, *};
use crate::index::entry::Entry;
use redb::{ReadableMultimapTable, ReadableTable};
use std::collections::BTreeMap;

pub struct Wjk20Updater<'wtx> {
  tables: Wjk20WriteTables<'wtx>,
  network: Network,
}

impl<'wtx> Wjk20Updater<'wtx> {
  pub fn new(tables: Wjk20WriteTables<'wtx>, network: Network) -> Self {
    Self { tables, network }
  }

  pub fn mark_wojakmap_root(&mut self, parent_number: u32) -> Result {
    if self
      .tables
      .wojakmap_roots
      .get(&parent_number)?
      .is_none()
    {
      self.tables.wojakmap_roots.insert(&parent_number, &1u8)?;
    }
    Ok(())
  }

  /// Dogemap-style: first `{N}.wojakmap` body claims block N if N <= inscription height.
  pub fn try_claim_wojakmap(
    &mut self,
    inscription_id: InscriptionId,
    height: u32,
    timestamp: u32,
    body: Option<&[u8]>,
  ) -> Result {
    let Some(body) = body else {
      return Ok(());
    };
    let Some(target_block) = parse::parse_wojakmap_claim(body) else {
      return Ok(());
    };
    if target_block > height {
      return Ok(());
    }
    if self.tables.wojakmap_claims.get(&target_block)?.is_some() {
      return Ok(());
    }
    let entry = crate::index::WojakmapClaimEntry {
      block_number: target_block,
      owner_inscription_id: inscription_id,
      claim_height: height,
      claim_timestamp: timestamp,
    };
    self
      .tables
      .wojakmap_claims
      .insert(&target_block, entry.store())?;
    Ok(())
  }

  pub fn on_inscription_revealed(
    &mut self,
    inscription_id: InscriptionId,
    inscription_number: u32,
    height: u32,
    timestamp: u32,
    body: Option<&[u8]>,
    content_type: Option<&str>,
  ) -> Result {
    self.try_claim_wojakmap(inscription_id, height, timestamp, body)?;
    self.try_register_domain(
      inscription_id,
      inscription_number,
      height,
      timestamp,
      body,
      content_type,
    )?;

    let Some(body) = body else {
      return Ok(());
    };
    let Some(parsed) = parse::parse_body(body) else {
      return Ok(());
    };

    match parsed.op {
      Wjk20Op::Deploy => self.apply_deploy(inscription_id, inscription_number, height, &parsed),
      Wjk20Op::Mint => self.stage_mint(inscription_id, inscription_number, height, &parsed),
      Wjk20Op::Transfer => {
        self.stage_transfer(inscription_id, inscription_number, height, &parsed)
      }
    }
  }

  /// Dash-style: first `{label}.wjk` text inscription registers that name.
  pub fn try_register_domain(
    &mut self,
    inscription_id: InscriptionId,
    inscription_number: u32,
    height: u32,
    timestamp: u32,
    body: Option<&[u8]>,
    content_type: Option<&str>,
  ) -> Result {
    let Some(body) = body else {
      return Ok(());
    };
    let Some(name) = parse::parse_wjk_domain(body, content_type) else {
      return Ok(());
    };
    if self.tables.domain_name.get(name.as_str())?.is_some() {
      return Ok(());
    }
    let record = DomainRecord {
      name: name.clone(),
      full_name: format!("{name}.wjk"),
      inscription_id: inscription_id.to_string(),
      inscription_number,
      owner_address: String::new(),
      height,
      timestamp,
    };
    let bytes = serde_json::to_vec(&record)?;
    self
      .tables
      .domain_name
      .insert(name.as_str(), bytes.as_slice())?;
    self
      .tables
      .domain_id_to_name
      .insert(&inscription_id.store(), name.as_str())?;
    Ok(())
  }

  fn set_domain_owner(&mut self, inscription_id: InscriptionId, address: &str) -> Result {
    let id = inscription_id.store();
    let Some(name_guard) = self.tables.domain_id_to_name.get(&id)? else {
      return Ok(());
    };
    let name = name_guard.value().to_string();
    drop(name_guard);

    let Some(record_guard) = self.tables.domain_name.get(name.as_str())? else {
      return Ok(());
    };
    let mut record: DomainRecord = serde_json::from_slice(record_guard.value())?;
    drop(record_guard);

    let old_owner = record.owner_address.clone();
    if old_owner == address {
      return Ok(());
    }

    if !old_owner.is_empty() {
      let _ = self
        .tables
        .domain_address_to_names
        .remove(old_owner.as_str(), name.as_str())?;
    }

    record.owner_address = address.to_string();
    let bytes = serde_json::to_vec(&record)?;
    self
      .tables
      .domain_name
      .insert(name.as_str(), bytes.as_slice())?;

    if !address.is_empty() && address != "unknown" && address != "unbound" {
      self
        .tables
        .domain_address_to_names
        .insert(address, name.as_str())?;
    }
    Ok(())
  }

  pub fn on_inscription_placed(
    &mut self,
    inscription_id: InscriptionId,
    inscription_number: u32,
    address: &str,
  ) -> Result {
    self.set_domain_owner(inscription_id, address)?;
    self.patch_event(inscription_number, |event| {
      event.address = Some(address.to_string());
    })?;
    self.set_deploy_deployer(&inscription_id, address)?;

    let id = inscription_id.store_bytes();
    let pending: PendingMint = {
      let Some(guard) = self.tables.pending_mint.get(&id)? else {
        return Ok(());
      };
      serde_json::from_slice(guard.value())?
    };
    self.tables.pending_mint.remove(&id)?;
    self.credit(&pending.tick, address, pending.amt)?;
    Ok(())
  }

  pub fn on_inscription_sent(
    &mut self,
    inscription_id: InscriptionId,
    inscription_number: u32,
    from_address: &str,
    to_address: &str,
  ) -> Result {
    if from_address == to_address || from_address == "unbound" || to_address == "unbound" {
      return Ok(());
    }

    self.set_domain_owner(inscription_id, to_address)?;
    let id = inscription_id.store_bytes();
    let pending: PendingTransfer = {
      let Some(guard) = self.tables.pending_transfer.get(&id)? else {
        return Ok(());
      };
      serde_json::from_slice(guard.value())?
    };

    self.debit(&pending.tick, from_address, pending.amt)?;
    self.credit(&pending.tick, to_address, pending.amt)?;
    self.tables.pending_transfer.remove(&id)?;
    self.patch_event(inscription_number, |event| {
      event.address = Some(from_address.to_string());
      event.to_address = Some(to_address.to_string());
    })?;
    Ok(())
  }

  pub fn replay_block_data(
    &mut self,
    height: u32,
    block: &crate::index::updater::BlockData,
    inscription_number: impl Fn(InscriptionId) -> Option<u32>,
  ) -> Result {
    let mut next_number = 0u32;
    let mut satpoint_to_id: BTreeMap<SatPoint, InscriptionId> = BTreeMap::new();
    let mut id_to_address: BTreeMap<InscriptionId, String> = BTreeMap::new();
    let timestamp = block.header.time;

    for (tx, txid) in block.txdata.iter().skip(1).chain(block.txdata.first()) {
      self.replay_transaction(
        tx,
        *txid,
        height,
        timestamp,
        &mut next_number,
        &inscription_number,
        &mut satpoint_to_id,
        &mut id_to_address,
      )?;
    }
    Ok(())
  }

  fn replay_transaction(
    &mut self,
    tx: &Transaction,
    txid: Txid,
    height: u32,
    timestamp: u32,
    next_number: &mut u32,
    inscription_number: &impl Fn(InscriptionId) -> Option<u32>,
    satpoint_to_id: &mut BTreeMap<SatPoint, InscriptionId>,
    id_to_address: &mut BTreeMap<InscriptionId, String>,
  ) -> Result {
    let mut inscriptions: Vec<(InscriptionId, u64, bool)> = Vec::new();
    let mut input_value = 0u64;

    for tx_in in &tx.input {
      if tx_in.previous_output.is_null() {
        input_value += Height(height).subsidy();
      } else {
        let old_satpoint = SatPoint {
          outpoint: tx_in.previous_output,
          offset: 0,
        };
        if let Some(id) = satpoint_to_id.remove(&old_satpoint) {
          if let Some(from) = id_to_address.get(&id).cloned() {
            inscriptions.push((id, input_value, false));
            let _ = from;
          }
        }
        input_value += 1;
      }
    }

    let txs = vec![tx.clone()];
    let mut new_id = None;
    if let ParsedInscription::Complete(inscription) = Inscription::from_transactions(&txs) {
      let id = InscriptionId { txid, index: 0 };
      let number = inscription_number(id).unwrap_or(*next_number);
      *next_number = (*next_number).max(number.saturating_add(1));
      new_id = Some((id, inscription, number));
      inscriptions.push((id, 0, true));
    }

    let mut output_value = 0u64;
    let mut flotsam_iter = inscriptions.into_iter().peekable();
    for (vout, tx_out) in tx.output.iter().enumerate() {
      let end = output_value + tx_out.value;
      while let Some((id, offset, is_new)) = flotsam_iter.peek().copied() {
        if offset >= end {
          break;
        }
        flotsam_iter.next();
        let address = Address::from_script(&tx_out.script_pubkey, self.network)
          .map(|a| a.to_string())
          .unwrap_or_else(|_| "unknown".to_string());

        if is_new {
          if let Some((new_id, ref inscription, number)) = new_id {
            if id == new_id {
              self.on_inscription_revealed(
                new_id,
                number,
                height,
                timestamp,
                inscription.body.as_deref(),
                inscription.content_type(),
              )?;
              self.on_inscription_placed(new_id, number, &address)?;
            }
          }
          id_to_address.insert(id, address.clone());
        } else if let Some(from) = id_to_address.get(&id).cloned() {
          if let Some(number) = inscription_number_for_id(&self.tables, &id)? {
            self.on_inscription_sent(id, number, &from, &address)?;
          }
          id_to_address.insert(id, address.clone());
        }

        let satpoint = SatPoint {
          outpoint: OutPoint {
            txid,
            vout: vout.try_into().unwrap(),
          },
          offset: offset.saturating_sub(output_value),
        };
        satpoint_to_id.insert(satpoint, id);
      }
      output_value = end;
    }

    Ok(())
  }

  fn apply_deploy(
    &mut self,
    inscription_id: InscriptionId,
    inscription_number: u32,
    height: u32,
    parsed: &ParsedWjk20,
  ) -> Result {
    if self.tables.deploy.get(parsed.tick.as_str())?.is_some() {
      return Ok(());
    }

    let Some(max) = parsed.max else {
      return Ok(());
    };
    let Some(lim) = parsed.lim else {
      return Ok(());
    };
    if max == 0 || lim == 0 || max < lim {
      return Ok(());
    }

    let record = DeployRecord {
      tick: parsed.tick.clone(),
      max: amount_to_string(max),
      lim: amount_to_string(lim),
      dec: parsed.dec.unwrap_or(18),
      inscription_id: inscription_id.to_string(),
      inscription_number,
      height,
      deployer: String::new(),
    };

    let bytes = serde_json::to_vec(&record)?;
    self
      .tables
      .deploy
      .insert(parsed.tick.as_str(), bytes.as_slice())?;

    self.store_event(
      inscription_number,
      &parsed.tick,
      EventRecord {
        op: "deploy".into(),
        tick: parsed.tick.clone(),
        amt: None,
        max: Some(amount_to_string(max)),
        lim: Some(amount_to_string(lim)),
        inscription_id: inscription_id.to_string(),
        inscription_number,
        height,
        address: None,
        to_address: None,
      },
    )?;

    Ok(())
  }

  fn stage_mint(
    &mut self,
    inscription_id: InscriptionId,
    inscription_number: u32,
    height: u32,
    parsed: &ParsedWjk20,
  ) -> Result {
    let Some(amt) = parsed.amt else {
      return Ok(());
    };
    if !self.deploy_exists(&parsed.tick)? {
      return Ok(());
    }
    let deploy = self.get_deploy(&parsed.tick)?;
    let lim: u128 = deploy.lim.parse()?;
    let max: u128 = deploy.max.parse()?;
    if amt == 0 || amt > lim {
      return Ok(());
    }
    let minted = self.total_minted(&parsed.tick)?;
    if minted.saturating_add(amt) > max {
      return Ok(());
    }

    let pending = PendingMint {
      tick: parsed.tick.clone(),
      amt,
    };
    self.tables.pending_mint.insert(
      &inscription_id.store_bytes(),
      serde_json::to_vec(&pending)?.as_slice(),
    )?;

    self.store_event(
      inscription_number,
      &parsed.tick,
      EventRecord {
        op: "mint".into(),
        tick: parsed.tick.clone(),
        amt: Some(amount_to_string(amt)),
        max: None,
        lim: None,
        inscription_id: inscription_id.to_string(),
        inscription_number,
        height,
        address: None,
        to_address: None,
      },
    )?;

    Ok(())
  }

  fn stage_transfer(
    &mut self,
    inscription_id: InscriptionId,
    inscription_number: u32,
    height: u32,
    parsed: &ParsedWjk20,
  ) -> Result {
    let Some(amt) = parsed.amt else {
      return Ok(());
    };
    if amt == 0 || !self.deploy_exists(&parsed.tick)? {
      return Ok(());
    }

    let pending = PendingTransfer {
      tick: parsed.tick.clone(),
      amt,
    };
    self.tables.pending_transfer.insert(
      &inscription_id.store_bytes(),
      serde_json::to_vec(&pending)?.as_slice(),
    )?;

    self.store_event(
      inscription_number,
      &parsed.tick,
      EventRecord {
        op: "transfer".into(),
        tick: parsed.tick.clone(),
        amt: Some(amount_to_string(amt)),
        max: None,
        lim: None,
        inscription_id: inscription_id.to_string(),
        inscription_number,
        height,
        address: None,
        to_address: None,
      },
    )?;

    Ok(())
  }

  fn store_event(
    &mut self,
    inscription_number: u32,
    tick: &str,
    event: EventRecord,
  ) -> Result {
    let bytes = serde_json::to_vec(&event)?;
    self
      .tables
      .event_by_number
      .insert(&inscription_number, bytes.as_slice())?;
    self
      .tables
      .tick_to_numbers
      .insert(tick, &inscription_number)?;
    Ok(())
  }

  fn deploy_exists(&self, tick: &str) -> Result<bool> {
    Ok(self.tables.deploy.get(tick)?.is_some())
  }

  fn get_deploy(&self, tick: &str) -> Result<DeployRecord> {
    let guard = self.tables.deploy.get(tick)?.unwrap();
    Ok(serde_json::from_slice(guard.value())?)
  }

  fn total_minted(&self, tick: &str) -> Result<u128> {
    let deploy = self.get_deploy(tick)?;
    let max: u128 = deploy.max.parse()?;
    let mut minted = 0u128;
    for number in self.tables.tick_to_numbers.get(tick)? {
      let number = number?.value();
      let Some(event) = self.tables.event_by_number.get(&number)? else {
        continue;
      };
      let event: EventRecord = serde_json::from_slice(event.value())?;
      if event.op == "mint" {
        if let Some(amt) = event.amt.as_ref().and_then(|s| s.parse().ok()) {
          minted = minted.saturating_add(amt);
        }
      }
    }
    let _ = max;
    Ok(minted)
  }

  fn balance_of(&self, tick: &str, address: &str) -> Result<u128> {
    let key = balance_key(tick, address);
    Ok(
      self
        .tables
        .balance
        .get(key.as_slice())?
        .map(|v| parse_balance(v.value()))
        .transpose()?
        .unwrap_or(0),
    )
  }

  fn credit(&mut self, tick: &str, address: &str, amt: u128) -> Result {
    let current = self.balance_of(tick, address)?;
    let new_balance = current.saturating_add(amt);
    let key = balance_key(tick, address);
    self.tables.balance.insert(
      key.as_slice(),
      amount_to_string(new_balance).as_bytes(),
    )?;
    Ok(())
  }

  fn patch_event(
    &mut self,
    inscription_number: u32,
    update: impl FnOnce(&mut EventRecord),
  ) -> Result {
    let Some(guard) = self.tables.event_by_number.get(&inscription_number)? else {
      return Ok(());
    };
    let mut event: EventRecord = serde_json::from_slice(guard.value())?;
    update(&mut event);
    drop(guard);
    let bytes = serde_json::to_vec(&event)?;
    self
      .tables
      .event_by_number
      .insert(&inscription_number, bytes.as_slice())?;
    Ok(())
  }

  fn set_deploy_deployer(&mut self, inscription_id: &InscriptionId, deployer: &str) -> Result {
    let id_str = inscription_id.to_string();
    let mut tick_to_update = None;
    for entry in self.tables.deploy.iter()? {
      let (tick, value) = entry?;
      let record: DeployRecord = serde_json::from_slice(value.value())?;
      if record.inscription_id == id_str {
        tick_to_update = Some((tick.value().to_string(), record));
        break;
      }
    }
    if let Some((tick, mut record)) = tick_to_update {
      record.deployer = deployer.to_string();
      let bytes = serde_json::to_vec(&record)?;
      self.tables.deploy.insert(tick.as_str(), bytes.as_slice())?;
    }
    Ok(())
  }

  fn debit(&mut self, tick: &str, address: &str, amt: u128) -> Result {
    let current = self.balance_of(tick, address)?;
    if current < amt {
      log::warn!(
        "WJK-20 transfer skipped: insufficient {tick} balance for {address} (have {current}, need {amt})"
      );
      return Ok(());
    }
    let new_balance = current - amt;
    let key = balance_key(tick, address);
    if new_balance == 0 {
      self.tables.balance.remove(key.as_slice())?;
    } else {
      self.tables.balance.insert(
        key.as_slice(),
        amount_to_string(new_balance).as_bytes(),
      )?;
    }
    Ok(())
  }
}

fn inscription_number_for_id(
  tables: &Wjk20WriteTables<'_>,
  inscription_id: &InscriptionId,
) -> Result<Option<u32>> {
  let id_str = inscription_id.to_string();
  for entry in tables.event_by_number.iter()? {
    let (_, value) = entry?;
    let event: EventRecord = serde_json::from_slice(value.value())?;
    if event.inscription_id == id_str {
      return Ok(Some(event.inscription_number));
    }
  }
  Ok(None)
}
