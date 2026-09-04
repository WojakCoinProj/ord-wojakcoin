use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Wjk20Op {
  Deploy,
  Mint,
  Transfer,
}

#[derive(Debug, Clone)]
pub struct ParsedWjk20 {
  pub op: Wjk20Op,
  pub tick: String,
  pub amt: Option<u128>,
  pub max: Option<u128>,
  pub lim: Option<u128>,
  pub dec: Option<u8>,
}

pub fn parse_body(body: &[u8]) -> Option<ParsedWjk20> {
  let value: serde_json::Value = serde_json::from_slice(body).ok()?;
  let obj = value.as_object()?;

  let protocol = obj.get("p")?.as_str()?.to_ascii_lowercase();
  if protocol != "wjk-20" && protocol != "wjk20" {
    return None;
  }

  let op = obj.get("op")?.as_str()?.to_ascii_lowercase();
  let op = match op.as_str() {
    "deploy" => Wjk20Op::Deploy,
    "mint" => Wjk20Op::Mint,
    "transfer" => Wjk20Op::Transfer,
    _ => return None,
  };

  let tick = obj.get("tick")?.as_str()?.to_string();
  if !valid_tick(&tick) {
    return None;
  }

  let amt = obj.get("amt").and_then(parse_amount);
  let max = obj.get("max").and_then(parse_amount);
  let lim = obj
    .get("lim")
    .and_then(parse_amount)
    .or_else(|| obj.get("limit").and_then(parse_amount));
  let dec = obj.get("dec").and_then(|v| {
    v.as_u64()
      .and_then(|n| u8::try_from(n).ok())
      .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
  });

  Some(ParsedWjk20 {
    op,
    tick: normalize_tick(&tick),
    amt,
    max,
    lim,
    dec,
  })
}

fn valid_tick(tick: &str) -> bool {
  let len = tick.len();
  (2..=8).contains(&len) && tick.bytes().all(|b| b.is_ascii_alphanumeric())
}

pub fn normalize_tick(tick: &str) -> String {
  tick.to_ascii_lowercase()
}

/// Parse dogemap-style body: trimmed UTF-8 exactly `{digits}.wojakmap`.
pub fn parse_wojakmap_claim(body: &[u8]) -> Option<u32> {
  let text = std::str::from_utf8(body).ok()?;
  let trimmed = text.trim();
  let prefix = trimmed.strip_suffix(".wojakmap")?;
  if prefix.is_empty() || !prefix.chars().all(|c| c.is_ascii_digit()) {
    return None;
  }
  prefix.parse().ok()
}

fn parse_amount(value: &serde_json::Value) -> Option<u128> {
  match value {
    serde_json::Value::String(s) => parse_amount_str(s),
    serde_json::Value::Number(n) => n.as_u64().map(u128::from),
    _ => None,
  }
}

fn parse_amount_str(s: &str) -> Option<u128> {
  if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
    return None;
  }
  s.parse().ok()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_deploy() {
    let body = br#"{"p":"wjk-20","op":"deploy","tick":"wojk","max":"21000000","lim":"1000"}"#;
    let p = parse_body(body).unwrap();
    assert_eq!(p.op, Wjk20Op::Deploy);
    assert_eq!(p.tick, "wojk");
    assert_eq!(p.max, Some(21_000_000));
    assert_eq!(p.lim, Some(1000));
  }

  #[test]
  fn parses_mint() {
    let body = br#"{"p":"wjk20","op":"mint","tick":"WOJK","amt":"500"}"#;
    let p = parse_body(body).unwrap();
    assert_eq!(p.op, Wjk20Op::Mint);
    assert_eq!(p.tick, "wojk");
    assert_eq!(p.amt, Some(500));
  }

  #[test]
  fn parses_wojakmap_claim() {
    assert_eq!(parse_wojakmap_claim(b"1.wojakmap"), Some(1));
    assert_eq!(parse_wojakmap_claim(b"  42.wojakmap\n"), Some(42));
    assert_eq!(parse_wojakmap_claim(b"1.Wojakmap"), None);
    assert_eq!(parse_wojakmap_claim(b"1.dogemap"), None);
    assert_eq!(parse_wojakmap_claim(b".wojakmap"), None);
    assert_eq!(parse_wojakmap_claim(b"01.wojakmap"), Some(1));
  }
}
