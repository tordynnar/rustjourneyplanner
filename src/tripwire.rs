use std::collections::HashMap;
use std::convert::From;
use chrono::{NaiveDateTime, Utc, Duration};
use web_sys;
use tracing::info;

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
    pub lifetime : NaiveDateTime,
    pub life : WormholeLife,
    pub mass : WormholeMass
}

#[derive(Debug, Clone)]
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

    let json : serde_json::Value = serde_json::from_slice(&result)
        .map_err(|_| format!("Tripwire JSON parse failed"))?;

    let signatures = match json["signatures"].as_object() {
        Some(s) => s,
        None => {
            let previous_result_value = previous_result.ok_or_else(|| format!("Tripwire signatures not present in initial refresh"))?;
            return Ok(TripwireRefresh { wormholes : vec![], signature_count : previous_result_value.signature_count, signature_time: previous_result_value.signature_time });
        }
    };

    let signature_time = signatures
        .iter().filter_map(|(_, v)| {
            v["modifiedTime"].as_str().and_then(|vv| NaiveDateTime::parse_from_str(vv, "%Y-%m-%d %H:%M:%S").ok())
        }).max().unwrap_or(NaiveDateTime::MIN);
    
    let signature_count = signatures.iter().count();

    info!("Signature update: {:?}", signature_time);

    let wormholes = json["wormholes"].as_object().ok_or_else(|| format!("Tripwire wormholes not present"))?;

    for (wormhole_id, wormhole) in wormholes {
        let initial_id = wormhole["initialID"].as_str().ok_or_else(|| format!("Tripwire initialID missing from wormhole {}", wormhole_id))?;
        let secondary_id = wormhole["secondaryID"].as_str().ok_or_else(|| format!("Tripwire secondaryID missing from wormhole {}", wormhole_id))?;

        let from_system = match json["signatures"][initial_id]["systemID"].as_str().and_then(|v| v.parse::<u32>().ok()) {
            Some(v) => v,
            None => continue
        };

        let to_system = SystemOrClass::from(json["signatures"][secondary_id]["systemID"].as_str().and_then(|v| v.parse::<u32>().ok()));

        let from_signature = json["signatures"][initial_id]["signatureID"].as_str().and_then(|v| match v { "???" => None, _ => Some(v.to_uppercase()) });
        let to_signature = json["signatures"][secondary_id]["signatureID"].as_str().and_then(|v| match v { "???" => None, _ => Some(v.to_uppercase()) });

        let wormhole_type = wormhole["type"].as_str().and_then(|v| match v { "????" => None, "" => None, _ => Some(v.to_owned()) });

        let lifetime_str = json["signatures"][initial_id]["lifeTime"].as_str().ok_or_else(|| format!("Tripwire wormhole lifeTime missing from {}", wormhole_id))?;
        let lifetime = NaiveDateTime::parse_from_str(lifetime_str, "%Y-%m-%d %H:%M:%S").map_err(|_| format!("Tripwire wormhole lifeTime wrong datetime format for {}", wormhole_id))?;
        let age = Utc::now().naive_utc() - lifetime;

        let life = match wormhole["life"].as_str() {
            Some("stable") => {
                if age < Duration::hours(20) {
                    Ok(WormholeLife::Stable)
                } else {
                    Ok(WormholeLife::EOL)
                }
            },
            Some("critical") => Ok(WormholeLife::EOL),
            Some(_) => Err(format!("Tripwire wormhole life is not stable or critical for {}", wormhole_id)),
            None => Err(format!("Tripwire wormhole life missing from {}", wormhole_id))
        }?;

        let mass = match wormhole["mass"].as_str() {
            Some("stable") => Ok(WormholeMass::Stable),
            Some("destab") => Ok(WormholeMass::Destab),
            Some("critical") => Ok(WormholeMass::VOC),
            Some(_) => Err(format!("Tripwire wormhole mass is not stable, destab or critical for {}", wormhole_id)),
            None => Err(format!("Tripwire wormhole mass is missing from {}", wormhole_id))
        }?;

        // Wormholes older than 24 hours probably don't exist any more
        if age > Duration::hours(24) { continue }

        // Probably created by a deathclone
        if from_signature == None && to_signature == None { continue }

        // Don't want gates, already have then in the static data
        if wormhole_type == Some("GATE".to_owned()) || from_signature == Some("GAT".to_owned()) || to_signature == Some("GAT".to_owned()) { continue }

        data.push(TripwireWormhole { from_system, to_system, from_signature, to_signature, wormhole_type, lifetime, life, mass });
    }

    Ok(TripwireRefresh {wormholes : data, signature_count, signature_time })
}
