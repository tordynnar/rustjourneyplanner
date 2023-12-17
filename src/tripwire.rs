use std::collections::HashMap;
use std::convert::From;
use chrono::{NaiveDateTime, Utc, Duration};
use web_sys;
use tracing::info;
use serde::{de::Error, Deserialize, Deserializer};
use serde_json;

fn deserialize_system_id<'de, D>(deserializer: D) -> Result<SystemOrClass, D::Error> where D: Deserializer<'de> {
    let s: Option<&str> = Deserialize::deserialize(deserializer)?;
    Ok(SystemOrClass::from(s.map(|v| v.parse::<u32>())
        .map_or(Ok(None), |v| v.map(Some))
        .map_err(D::Error::custom)?))
}

fn deserialize_signature_id<'de, D>(deserializer: D) -> Result<Option<String>, D::Error> where D: Deserializer<'de> {
    let s: Option<&str> = Deserialize::deserialize(deserializer)?;
    Ok(s.and_then(|v| match v { "???" => None, _ => Some(v.to_uppercase()) }))
}

fn deserialize_lifetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error> where D: Deserializer<'de> {
    let s: &str = Deserialize::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").map_err(D::Error::custom)
}

#[derive(Debug, Clone, Deserialize)]
pub struct TripwireWormholeRaw {
    #[serde(alias = "initialID")] initial_id : String,
    #[serde(alias = "secondaryID")] secondary_id : String,
    #[serde(alias = "type")] wormhole_type : Option<String>,
    life : String,
    mass : String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TripwireSignatureRaw {
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_system_id")]
    #[serde(alias = "systemID")]
    system_id : SystemOrClass,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_signature_id")]
    #[serde(alias = "signatureID")]
    signature_id : Option<String>,

    #[serde(deserialize_with = "deserialize_lifetime")]
    #[serde(alias = "lifeTime")] life_time : NaiveDateTime,

    #[serde(alias = "modifiedTime")] modified_time : String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TripwireRaw {
    pub signatures : Option<HashMap<String,TripwireSignatureRaw>>,
    pub wormholes : Option<HashMap<String,TripwireWormholeRaw>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WormholeLife {
    Stable,
    EOL
}

#[derive(Debug, Clone, PartialEq)]
pub enum WormholeMass {
    Stable,
    Destab,
    VOC
}

#[derive(Debug, Clone)]
pub struct TripwireWormhole {
    pub from_system : u32,
    pub to_system : SystemOrClass,
    pub from_signature : Option<String>,
    pub to_signature : Option<String>,
    pub wormhole_type : Option<String>,
    pub life_time : NaiveDateTime,
    pub life : WormholeLife,
    pub mass : WormholeMass
}

#[derive(Debug, Clone, Copy)]
pub enum SystemOrClass {
    SpecificSystem(u32),
    Nullsec,
    Lowsec,
    Highsec,
    Class1,
    Class2,
    Class3,
    Class4,
    Class5,
    Class6,
    Class13,
    Pochven,
    Unknown
}

impl Default for SystemOrClass {
    fn default() -> Self {
        SystemOrClass::Unknown
    }
}

impl From<Option<u32>> for SystemOrClass {
    fn from(item: Option<u32>) -> Self {
        match item {
            None => SystemOrClass::Unknown,
            Some(0) => SystemOrClass::Nullsec,
            Some(1) => SystemOrClass::Lowsec,
            Some(2) => SystemOrClass::Highsec,
            Some(3) => SystemOrClass::Class1,
            Some(4) => SystemOrClass::Class2,
            Some(5) => SystemOrClass::Class3,
            Some(6) => SystemOrClass::Class4,
            Some(7) => SystemOrClass::Class5,
            Some(8) => SystemOrClass::Class6,
            Some(9) => SystemOrClass::Class13,
            Some(10) => SystemOrClass::Pochven,
            Some(v) => SystemOrClass::SpecificSystem(v)
        }
    }
}

#[derive(Debug, Clone)]
pub struct TripwireRefresh {
    pub wormholes : Vec::<TripwireWormhole>,
    pub signature_count : usize,
    pub signature_time : NaiveDateTime,
}

impl PartialEq for TripwireRefresh {
    fn eq(&self, other: &TripwireRefresh) -> bool {
        self.signature_time.eq(&other.signature_time) && self.signature_count.eq(&other.signature_count)
    }
}

pub async fn get_tripwire(previous_result : Option<TripwireRefresh>) -> Result<TripwireRefresh, String> {
    let signature_count = previous_result.as_ref().map(|v| v.signature_count).unwrap_or(0);
    let signature_time = previous_result.as_ref().map(|v| v.signature_time).unwrap_or(NaiveDateTime::MIN);

    let mut data = Vec::<TripwireWormhole>::new();

    let baseurl = web_sys::window().ok_or_else(|| format!("Cannot get base URL"))?.origin();

    let client = reqwest::Client::new();
    let result = client.post(format!("{baseurl}/refresh.php"))
        .form(&HashMap::from([
            ("mode", "refresh".to_owned()),
            ("systemID", "30000142".to_owned()),
            ("systemName", "Jita".to_owned()),
            ("signatureCount", signature_count.to_string()),
            ("signatureTime", signature_time.format("%Y-%m-%d %H:%M:%S").to_string()),
        ]))
        .send().await.map_err(|_| format!("Tripwire HTTP request failed"))?
        .error_for_status().map_err(|_| format!("Tripwire HTTP request failed"))?
        .bytes().await.map_err(|_| format!("Tripwire HTTP request failed"))?;

    let json = serde_json::from_slice::<TripwireRaw>(&result)
        .map_err(|e| format!("Tripwire parse failed: {:?}", e))?;

    let signatures = match json.signatures {
        Some(s) => s,
        None => {
            let previous_result_value = previous_result.ok_or_else(|| format!("Tripwire signatures not present in initial refresh"))?;
            return Ok(TripwireRefresh { wormholes : vec![], signature_count : previous_result_value.signature_count, signature_time: previous_result_value.signature_time });
        }
    };

    let signature_time = signatures
        .iter()
        .map(|(_, v)| NaiveDateTime::parse_from_str(&v.modified_time, "%Y-%m-%d %H:%M:%S"))
        .collect::<Result<Vec<_>,_>>()
        .map_err(|_| format!("Tripwire modifiedTime format wrong"))?
        .into_iter()
        .max()
        .unwrap_or(NaiveDateTime::MIN);

    let signature_count = signatures.iter().count();

    info!("Signature update: {:?}", signature_time);

    let wormholes = json.wormholes.ok_or_else(|| format!("Tripwire wormholes not present"))?;

    for (wormhole_id, wormhole) in wormholes {
        let from = signatures.get(&wormhole.initial_id).ok_or_else(|| format!("Tripwire initial signature details missing from {}", wormhole_id))?;
        let to = signatures.get(&wormhole.secondary_id).ok_or_else(|| format!("Tripwire secondary signature details missing from {}", wormhole_id))?;

        /*
        let [(from_system, from_signature, from_life_time), (to_system, to_signature, to_life_time)] = [wormhole.initial_id, wormhole.secondary_id]
            .try_map(|v| signatures.get(&v))
            .ok_or_else(|| format!("Tripwire signature details missing from wormhole {}", wormhole_id))?
            .map(|signature| {
                (
                    signature.system_id.clone(),
                    signature.signature_id.clone(),
                    signature.life_time.clone()
                )
            });
        */

        let from_system = match from.system_id { SystemOrClass::SpecificSystem(v) => v, _ => continue };
        let wormhole_type = wormhole.wormhole_type.as_deref().and_then(|v| match v { "????" => None, "" => None, _ => Some(v.to_owned()) });
        let life_time = [from.life_time, to.life_time].into_iter().max().unwrap();
        let age = Utc::now().naive_utc() - life_time;

        let life = match wormhole.life.as_ref() {
            "stable" => {
                if age < Duration::hours(20) {
                    Ok(WormholeLife::Stable)
                } else {
                    Ok(WormholeLife::EOL)
                }
            },
            "critical" => Ok(WormholeLife::EOL),
            _ => Err(format!("Tripwire wormhole life is not stable or critical for {}", wormhole_id)),
        }?;

        let mass = match wormhole.mass.as_ref() {
            "stable" => Ok(WormholeMass::Stable),
            "destab" => Ok(WormholeMass::Destab),
            "critical" => Ok(WormholeMass::VOC),
            _ => Err(format!("Tripwire wormhole mass is not stable, destab or critical for {}", wormhole_id)),
        }?;

        // Wormholes older than 24 hours probably don't exist any more
        if age > Duration::hours(24) { continue }

        // Probably created by a deathclone
        if from.signature_id == None && to.signature_id == None { continue }

        // Don't want gates, already have then in the static data
        if wormhole_type == Some("GATE".to_owned()) || from.signature_id == Some("GAT".to_owned()) || to.signature_id == Some("GAT".to_owned()) { continue }

        data.push(TripwireWormhole { from_system, to_system : to.system_id, from_signature : from.signature_id.clone(), to_signature : to.signature_id.clone(), wormhole_type, life_time, life, mass });
    }

    Ok(TripwireRefresh {wormholes : data, signature_count, signature_time })
}
