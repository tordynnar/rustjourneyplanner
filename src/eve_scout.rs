/*
use serde::Deserialize;
use chrono::{NaiveDateTime, Utc};

#[derive(Debug, Clone, Deserialize)]
pub struct EveScoutWormhole {
    pub out_system_id : u32,
    pub out_signature : String,
    pub in_system_id : u32,
    pub in_signature : String,
    pub remaining_hours : u32,
    pub wh_type : String,
    pub updated_at : String
}

#[derive(Debug, Clone)]
pub struct EveScoutRefresh {
    pub wormholes : Vec::<EveScoutWormhole>,
    pub signature_count : usize,
    pub signature_time : NaiveDateTime,
    pub update_time : NaiveDateTime,
    pub update_error : Option<String>
}

impl PartialEq for EveScoutRefresh {
    pub fn eq(&self, other: &EveScoutRefresh) -> bool {
        self.signature_time.eq(&other.signature_time) && self.signature_count.eq(&other.signature_count)
    }
}

pub async fn get_eve_scout(_ : Option<EveScoutRefresh>) -> Result<EveScoutRefresh, String> {
    tracing::info!("EvE Scout updating...");

    let client = reqwest::Client::new();
    let result = client.get(format!("https://corsproxy.io/?{}", urlencoding::encode("https://api.eve-scout.com/v2/public/signatures")))
        .send().await.map_err(|_| format!("EvE-Scout HTTP request failed"))?
        .error_for_status().map_err(|_| format!("EvE-Scout HTTP request failed"))?
        .bytes().await.map_err(|_| format!("EvE-Scout HTTP request failed"))?;

    let wormholes = serde_json::from_slice::<Vec<EveScoutWormhole>>(&result)
        .map_err(|_| format!("EvE-Scout JSON parse failed"))?;

    let signature_count = wormholes.len();

    let signature_time = wormholes.iter()
        .map(|v| NaiveDateTime::parse_from_str(&v.updated_at, "%Y-%m-%dT%H:%M:%S.000Z"))
        .collect::<Result<Vec<_>,_>>()
        .map_err(|_| format!("EvE-Scout failed to parse wormhole updated_at"))?
        .into_iter()
        .max()
        .unwrap_or(NaiveDateTime::MIN);

    Ok(EveScoutRefresh { wormholes, signature_count, signature_time, update_time : Utc::now().naive_utc(), update_error: None })
}
*/