use super::*;

#[derive(Deserialize)]
pub(super) struct ListQuery {
  #[serde(default = "default_limit")]
  pub limit: usize,
  #[serde(default)]
  pub offset: usize,
}

fn default_limit() -> usize {
  100
}

pub(super) async fn api_tokens(
  Extension(index): Extension<Arc<Index>>,
) -> ServerResult<Json<Vec<api::Wjk20TokenInfo>>> {
  let tokens = index.wjk20_list_tokens()?;
  let out = tokens
    .into_iter()
    .map(|t| api::Wjk20TokenInfo {
      tick: t.tick,
      max: t.max,
      lim: t.lim,
      dec: t.dec,
      inscription_id: t.inscription_id,
      inscription_number: t.inscription_number,
      height: t.height,
      deployer: t.deployer,
    })
    .collect();
  Ok(Json(out))
}

pub(super) async fn api_token(
  Extension(index): Extension<Arc<Index>>,
  Path(tick): Path<String>,
) -> ServerResult {
  match index.wjk20_get_token(&tick)? {
    Some(t) => Ok(
      Json(api::Wjk20TokenInfo {
        tick: t.tick,
        max: t.max,
        lim: t.lim,
        dec: t.dec,
        inscription_id: t.inscription_id,
        inscription_number: t.inscription_number,
        height: t.height,
        deployer: t.deployer,
      })
      .into_response(),
    ),
    None => Ok(StatusCode::NOT_FOUND.into_response()),
  }
}

pub(super) async fn api_deploys(
  Extension(index): Extension<Arc<Index>>,
  Query(query): Query<ListQuery>,
) -> ServerResult<Json<Vec<api::Wjk20DeployEvent>>> {
  let events = index.wjk20_list_deploys(query.limit, query.offset)?;
  let out = events
    .into_iter()
    .map(|e| api::Wjk20DeployEvent {
      op: e.op,
      tick: e.tick,
      amt: e.amt,
      max: e.max,
      lim: e.lim,
      inscription_id: e.inscription_id,
      inscription_number: e.inscription_number,
      height: e.height,
      address: e.address,
      to_address: e.to_address,
    })
    .collect();
  Ok(Json(out))
}

pub(super) async fn api_balances(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> ServerResult<Json<Vec<api::Wjk20Balance>>> {
  Ok(Json(index.wjk20_balances(&address)?))
}

pub(super) async fn api_balance(
  Extension(index): Extension<Arc<Index>>,
  Path((address, tick)): Path<(String, String)>,
) -> ServerResult<Json<api::Wjk20Balance>> {
  let balance = index.wjk20_balance(&address, &tick)?;
  Ok(Json(api::Wjk20Balance {
    tick: crate::wjk20::normalize_tick(&tick),
    balance,
  }))
}

pub(super) async fn api_wojakmaps(
  Extension(index): Extension<Arc<Index>>,
  Query(query): Query<ListQuery>,
) -> ServerResult<Json<Vec<api::WojakmapClaim>>> {
  Ok(Json(index.wjk20_list_wojakmaps(query.limit, query.offset)?))
}

pub(super) async fn api_wojakmap_block(
  Extension(index): Extension<Arc<Index>>,
  Path(block_number): Path<u32>,
) -> ServerResult {
  match index.wojakmap_claim(block_number)? {
    Some(claim) => Ok(Json(claim).into_response()),
    None => Ok(StatusCode::NOT_FOUND.into_response()),
  }
}

pub(super) async fn api_wojakmap(
  Extension(index): Extension<Arc<Index>>,
  Path(inscription_id): Path<InscriptionId>,
) -> ServerResult {
  match index.wojakmap_claim_by_inscription(inscription_id)? {
    Some(claim) => Ok(Json(claim).into_response()),
    None => Ok(StatusCode::NOT_FOUND.into_response()),
  }
}

pub(super) async fn api_collections(
  Extension(index): Extension<Arc<Index>>,
  Query(query): Query<ListQuery>,
) -> ServerResult<Json<Vec<api::CollectionSummary>>> {
  Ok(Json(index.wjk20_list_collections(query.limit, query.offset)?))
}

pub(super) async fn api_collection(
  Extension(index): Extension<Arc<Index>>,
  Path(inscription_id): Path<InscriptionId>,
) -> ServerResult {
  match index.collection_detail(inscription_id)? {
    Some(detail) => Ok(Json(detail).into_response()),
    None => Ok(StatusCode::NOT_FOUND.into_response()),
  }
}

pub(super) async fn api_domains(
  Extension(index): Extension<Arc<Index>>,
  Query(query): Query<ListQuery>,
) -> ServerResult<Json<Vec<api::DomainInfo>>> {
  Ok(Json(index.list_domains(query.limit, query.offset)?))
}

pub(super) async fn api_domain_stats(
  Extension(index): Extension<Arc<Index>>,
) -> ServerResult<Json<api::DomainStats>> {
  Ok(Json(index.domain_stats()?))
}

pub(super) async fn api_domain_name(
  Extension(index): Extension<Arc<Index>>,
  Path(name): Path<String>,
) -> ServerResult<Json<api::DomainLookup>> {
  Ok(Json(index.domain_lookup(&name)?))
}

pub(super) async fn api_domains_by_address(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> ServerResult<Json<Vec<api::DomainInfo>>> {
  Ok(Json(index.domains_by_address(&address)?))
}
