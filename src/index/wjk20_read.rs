use super::*;
use redb::{ReadableMultimapTable, ReadableTable, ReadableTableMetadata};

impl Index {
  pub(crate) fn wjk20_list_tokens(&self) -> Result<Vec<crate::wjk20::DeployRecord>> {
    let rtx = self.database.begin_read()?;
    let tables = crate::wjk20::open_read_tables(&rtx)?;
    let mut tokens = Vec::new();
    for entry in tables.deploy.iter()? {
      let (_, value) = entry?;
      let bytes = value.value().to_vec();
      let record: crate::wjk20::DeployRecord = serde_json::from_slice(&bytes)?;
      tokens.push(record);
    }
    tokens.sort_by(|a, b| a.tick.cmp(&b.tick));
    Ok(tokens)
  }

  pub(crate) fn wjk20_get_token(&self, tick: &str) -> Result<Option<crate::wjk20::DeployRecord>> {
    let tick = crate::wjk20::parse::normalize_tick(tick);
    let rtx = self.database.begin_read()?;
    let tables = crate::wjk20::open_read_tables(&rtx)?;
    let Some(guard) = tables.deploy.get(tick.as_str())? else {
      return Ok(None);
    };
    let bytes = guard.value().to_vec();
    Ok(Some(serde_json::from_slice(&bytes)?))
  }

  pub(crate) fn wjk20_list_deploys(&self, limit: usize, offset: usize) -> Result<Vec<crate::wjk20::EventRecord>> {
    let rtx = self.database.begin_read()?;
    let tables = crate::wjk20::open_read_tables(&rtx)?;
    let mut events = Vec::new();
    for entry in tables.event_by_number.iter()? {
      let (_, value) = entry?;
      let event: crate::wjk20::EventRecord = serde_json::from_slice(value.value())?;
      if event.op == "deploy" {
        events.push(event);
      }
    }
    events.sort_by(|a, b| b.height.cmp(&a.height));
    Ok(events.into_iter().skip(offset).take(limit).collect())
  }

  pub(crate) fn wjk20_balance(&self, address: &str, tick: &str) -> Result<String> {
    let tick = crate::wjk20::parse::normalize_tick(tick);
    let rtx = self.database.begin_read()?;
    let tables = crate::wjk20::open_read_tables(&rtx)?;
    let key = crate::wjk20::balance_key(&tick, address);
    let balance = tables
      .balance
      .get(key.as_slice())?
      .map(|v| String::from_utf8_lossy(v.value()).into_owned())
      .unwrap_or_else(|| "0".into());
    Ok(balance)
  }

  pub(crate) fn wjk20_balances(&self, address: &str) -> Result<Vec<api::Wjk20Balance>> {
    let rtx = self.database.begin_read()?;
    let tables = crate::wjk20::open_read_tables(&rtx)?;
    let mut balances = Vec::new();
    for entry in tables.balance.iter()? {
      let (key, value) = entry?;
      let key = key.value();
      let Some(nul) = key.iter().position(|&b| b == 0) else {
        continue;
      };
      let tick = std::str::from_utf8(&key[..nul])?;
      let holder = std::str::from_utf8(&key[nul + 1..])?;
      if holder != address {
        continue;
      }
      balances.push(api::Wjk20Balance {
        tick: tick.to_string(),
        balance: String::from_utf8_lossy(value.value()).into_owned(),
      });
    }
    balances.sort_by(|a, b| a.tick.cmp(&b.tick));
    Ok(balances)
  }

  pub(crate) fn wjk20_list_wojakmaps(&self, limit: usize, offset: usize) -> Result<Vec<api::WojakmapClaim>> {
    let rtx = self.database.begin_read()?;
    let tables = crate::wjk20::open_read_tables(&rtx)?;
    let mut claims = Vec::new();
    for entry in tables.wojakmap_claims.iter()? {
      let (_, value) = entry?;
      let claim = crate::index::WojakmapClaimEntry::load(value.value());
      claims.push(api::WojakmapClaim {
        block_number: claim.block_number,
        inscription_id: claim.owner_inscription_id,
        claim_height: claim.claim_height,
        claim_timestamp: claim.claim_timestamp,
      });
    }
    claims.sort_by(|a, b| a.block_number.cmp(&b.block_number));
    Ok(claims.into_iter().skip(offset).take(limit).collect())
  }

  pub(crate) fn wojakmap_claim(&self, block_number: u32) -> Result<Option<api::WojakmapClaim>> {
    let rtx = self.database.begin_read()?;
    let tables = crate::wjk20::open_read_tables(&rtx)?;
    let Some(guard) = tables.wojakmap_claims.get(&block_number)? else {
      return Ok(None);
    };
    let claim = crate::index::WojakmapClaimEntry::load(guard.value());
    Ok(Some(api::WojakmapClaim {
      block_number: claim.block_number,
      inscription_id: claim.owner_inscription_id,
      claim_height: claim.claim_height,
      claim_timestamp: claim.claim_timestamp,
    }))
  }

  pub(crate) fn wojakmap_claim_by_inscription(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<api::WojakmapClaim>> {
    let rtx = self.database.begin_read()?;
    let tables = crate::wjk20::open_read_tables(&rtx)?;
    for entry in tables.wojakmap_claims.iter()? {
      let (_, value) = entry?;
      let claim = crate::index::WojakmapClaimEntry::load(value.value());
      if claim.owner_inscription_id == inscription_id {
        return Ok(Some(api::WojakmapClaim {
          block_number: claim.block_number,
          inscription_id: claim.owner_inscription_id,
          claim_height: claim.claim_height,
          claim_timestamp: claim.claim_timestamp,
        }));
      }
    }
    Ok(None)
  }

  pub(crate) fn wjk20_list_collections(&self, limit: usize, offset: usize) -> Result<Vec<api::CollectionSummary>> {
    let rtx = self.database.begin_read()?;
    let tables = crate::wjk20::open_read_tables(&rtx)?;
    let mut maps = Vec::new();
    for entry in tables.wojakmap_roots.iter()? {
      let (number, _) = entry?;
      let number = number.value();
      if let Some(summary) = self.collection_summary(number)? {
        maps.push(summary);
      }
    }
    maps.sort_by(|a, b| b.child_count.cmp(&a.child_count));
    Ok(maps.into_iter().skip(offset).take(limit).collect())
  }

  pub(crate) fn collection_summary(&self, parent_number: u32) -> Result<Option<api::CollectionSummary>> {
    let parent_id = self.get_inscription_id_by_inscription_number(parent_number)?;
    let Some(parent_id) = parent_id else {
      return Ok(None);
    };
    let children = self.get_children_by_number(parent_number)?;
    let properties = self
      .get_inscription_by_id(parent_id)?
      .and_then(|i| i.properties())
      .map(api::Properties::from);
    Ok(Some(api::CollectionSummary {
      inscription_id: parent_id,
      inscription_number: parent_number,
      child_count: children.len() as u64,
      properties,
    }))
  }

  pub(crate) fn collection_detail(&self, inscription_id: InscriptionId) -> Result<Option<api::CollectionDetail>> {
    let entry = self.get_inscription_entry(inscription_id)?;
    let Some(entry) = entry else {
      return Ok(None);
    };
    let children = self.get_children_by_number(entry.number)?;
    let properties = self
      .get_inscription_by_id(inscription_id)?
      .and_then(|i| i.properties())
      .map(api::Properties::from);
    Ok(Some(api::CollectionDetail {
      inscription_id,
      inscription_number: entry.number,
      children,
      properties,
    }))
  }

  pub(crate) fn get_children_by_number(&self, number: u32) -> Result<Vec<InscriptionId>> {
    let rtx = self.database.begin_read()?;
    let children_table = rtx.open_multimap_table(INSCRIPTION_NUMBER_TO_CHILDREN)?;
    let id_table = rtx.open_table(INSCRIPTION_NUMBER_TO_INSCRIPTION_ID)?;
    let mut children = Vec::new();
    for child_number in children_table.get(number)? {
      let child_number = child_number?.value();
      if let Some(id) = id_table.get(&child_number)? {
        children.push(InscriptionId::load(*id.value()));
      }
    }
    Ok(children)
  }

  fn domain_record_to_api(record: crate::wjk20::tables::DomainRecord) -> api::DomainInfo {
    api::DomainInfo {
      name: record.name,
      full_name: record.full_name,
      inscription_id: record.inscription_id,
      inscription_number: record.inscription_number,
      owner_address: record.owner_address,
      height: record.height,
      timestamp: record.timestamp,
    }
  }

  pub(crate) fn list_domains(&self, limit: usize, offset: usize) -> Result<Vec<api::DomainInfo>> {
    let rtx = self.database.begin_read()?;
    let tables = crate::wjk20::open_read_tables(&rtx)?;
    let mut domains = Vec::new();
    for entry in tables.domain_name.iter()? {
      let (_, value) = entry?;
      let record: crate::wjk20::tables::DomainRecord = serde_json::from_slice(value.value())?;
      domains.push(Self::domain_record_to_api(record));
    }
    domains.sort_by(|a, b| b.height.cmp(&a.height).then_with(|| a.name.cmp(&b.name)));
    Ok(domains.into_iter().skip(offset).take(limit).collect())
  }

  pub(crate) fn get_domain(&self, name: &str) -> Result<Option<api::DomainInfo>> {
    let name = name
      .trim()
      .trim_end_matches(".wjk")
      .trim_end_matches(".WJK")
      .to_ascii_lowercase();
    let rtx = self.database.begin_read()?;
    let tables = crate::wjk20::open_read_tables(&rtx)?;
    let Some(guard) = tables.domain_name.get(name.as_str())? else {
      return Ok(None);
    };
    let record: crate::wjk20::tables::DomainRecord = serde_json::from_slice(guard.value())?;
    Ok(Some(Self::domain_record_to_api(record)))
  }

  pub(crate) fn domain_lookup(&self, name: &str) -> Result<api::DomainLookup> {
    let normalized = name
      .trim()
      .trim_end_matches(".wjk")
      .trim_end_matches(".WJK")
      .to_ascii_lowercase();
    let full_name = format!("{normalized}.wjk");
    let domain = self.get_domain(&normalized)?;
    Ok(api::DomainLookup {
      name: normalized,
      full_name,
      available: domain.is_none(),
      domain,
    })
  }

  pub(crate) fn domains_by_address(&self, address: &str) -> Result<Vec<api::DomainInfo>> {
    let rtx = self.database.begin_read()?;
    let tables = crate::wjk20::open_read_tables(&rtx)?;
    let mut domains = Vec::new();
    for name in tables.domain_address_to_names.get(address)? {
      let name = name?.value().to_string();
      if let Some(guard) = tables.domain_name.get(name.as_str())? {
        let record: crate::wjk20::tables::DomainRecord = serde_json::from_slice(guard.value())?;
        domains.push(Self::domain_record_to_api(record));
      }
    }
    domains.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(domains)
  }

  pub(crate) fn domain_stats(&self) -> Result<api::DomainStats> {
    let rtx = self.database.begin_read()?;
    let tables = crate::wjk20::open_read_tables(&rtx)?;
    let mut owners = std::collections::BTreeSet::new();
    let mut total = 0u64;
    for entry in tables.domain_name.iter()? {
      let (_, value) = entry?;
      let record: crate::wjk20::tables::DomainRecord = serde_json::from_slice(value.value())?;
      total += 1;
      if !record.owner_address.is_empty() {
        owners.insert(record.owner_address);
      }
    }
    Ok(api::DomainStats {
      total,
      unique_owners: owners.len() as u64,
    })
  }
}
