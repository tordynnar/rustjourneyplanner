use serde::Deserialize;
use chrono::{NaiveDateTime, Utc};

#[derive(Debug, Clone, Deserialize)]
pub struct EveScoutWormhole {
    out_system_id : u32,
    out_signature : String,
    in_system_id : u32,
    in_signature : String,
    remaining_hours : u32,
    wh_type : String,
    updated_at : String
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
    fn eq(&self, other: &EveScoutRefresh) -> bool {
        self.signature_time.eq(&other.signature_time) && self.signature_count.eq(&other.signature_count)
    }
}

pub async fn get_eve_scout() -> Result<EveScoutRefresh, String> {
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

pub async fn get_eve_scout_memoable(previous_result : &Option<Result<EveScoutRefresh, String>>) -> Result<EveScoutRefresh, String> {
    let previous_result = previous_result.as_ref().map(|v| v.as_ref()).map_or_else(|| None, |v| v.ok());

    let result = match get_eve_scout().await {
        Ok(v) => v,
        Err(e) => {
            let previous_result_value = previous_result.ok_or_else(|| e.clone())?;
            EveScoutRefresh { wormholes : vec![], signature_count : previous_result_value.signature_count, signature_time: previous_result_value.signature_time, update_time : previous_result_value.update_time, update_error : Some(e) }
        }
    };

    Ok(result)
}
