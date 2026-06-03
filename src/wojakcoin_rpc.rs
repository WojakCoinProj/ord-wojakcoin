use {
  bitcoin::{consensus, BlockHash, Transaction, Txid},
  bitcoincore_rpc::{Client, Error, RpcApi},
};

/// Wojakcoin Core uses numeric verbosity for `getrawtransaction` (0 = hex, 1 = JSON),
/// unlike Bitcoin Core's boolean. The dogecoin RPC client sends booleans and fails.
pub(crate) fn get_raw_transaction(client: &Client, txid: &Txid) -> Result<Transaction, Error> {
  let hex: String = client.call(
    "getrawtransaction",
    &[serde_json::to_value(txid).map_err(Error::Json)?, serde_json::json!(0)],
  )?;
  let bytes = hex::decode(&hex).map_err(|err| Error::ReturnedError(err.to_string()))?;
  consensus::deserialize(&bytes).map_err(Error::BitcoinSerialization)
}

pub(crate) fn get_raw_transaction_blockhash(
  client: &Client,
  txid: &Txid,
) -> Result<Option<BlockHash>, Error> {
  let tx: serde_json::Value = client.call(
    "getrawtransaction",
    &[serde_json::to_value(txid).map_err(Error::Json)?, serde_json::json!(1)],
  )?;

  let Some(hash) = tx.get("blockhash").and_then(|value| value.as_str()) else {
    return Ok(None);
  };

  Ok(Some(
    hash
      .parse::<BlockHash>()
      .map_err(|_| Error::UnexpectedStructure)?,
  ))
}

pub(crate) fn get_confirmations(client: &Client, txid: &Txid) -> Result<Option<u32>, Error> {
  let tx: serde_json::Value = client.call(
    "getrawtransaction",
    &[serde_json::to_value(txid).map_err(Error::Json)?, serde_json::json!(1)],
  )?;

  Ok(
    tx.get("confirmations")
      .and_then(|value| value.as_u64())
      .map(|confirmations| confirmations as u32),
  )
}

pub(crate) fn is_transaction_in_active_chain(client: &Client, txid: &Txid) -> Result<bool, Error> {
  let tx: serde_json::Value = client.call(
    "getrawtransaction",
    &[serde_json::to_value(txid).map_err(Error::Json)?, serde_json::json!(1)],
  )?;

  Ok(
    tx.get("blockhash")
      .and_then(|value| value.as_str())
      .is_some()
      && tx.get("confirmations")
        .and_then(|value| value.as_u64())
        .unwrap_or(0)
        > 0,
  )
}
