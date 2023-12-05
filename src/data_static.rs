use std::cmp::Ordering;
use std::collections::HashMap;
use web_sys;

#[derive(Debug, Clone)]
pub struct System {
    pub id : u32,
    pub name: String,
    pub security: f32,
    pub class: Option<u16>
}

impl PartialEq for System {
    fn eq(&self, other: &System) -> bool {
        self.id == other.id
    }
}

impl Eq for System {}

impl PartialOrd for System {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for System {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Debug, Clone)]
pub struct Gate {
    pub from_system : u32,
    pub to_system: u32
}

#[derive(Debug, Clone)]
pub struct StaticData {
    pub systems : Vec::<System>,
    pub gates : Vec::<Gate>,
    pub wormhole_jump_mass : HashMap::<String,u32>
}

pub async fn get_static_data() -> Result<StaticData, String> {
    let mut systems = Vec::<System>::new();
    let mut gates = Vec::<Gate>::new();
    let mut wormhole_jump_mass = HashMap::<String,u32>::new();

    let baseurl = web_sys::window().ok_or_else(|| format!("Cannot get base URL"))?.origin();

    let result = reqwest::get(format!("{baseurl}/js/combine.js")).await
        .map_err(|_| format!("Failed to send request for combine.js"))?
        .error_for_status().map_err(|_| format!("Bad status code getting combine.js"))?
        .bytes().await
        .map_err(|_| format!("Failed to get bytes for combine.js"))?;

    let json : serde_json::Value = serde_json::from_slice(&result[14..])
        .map_err(|_| format!("Failed to parse combine.js JSON"))?;

    let systems_data = json["systems"].as_object().ok_or_else(|| format!("Systems missing from combine.js"))?;
    let gates_data = json["map"]["shortest"].as_object().ok_or_else(|| format!("Map missing from combine.js"))?;
    let wormhole_data = json["wormholes"].as_object().ok_or_else(|| format!("Wormholes missing from combine.js"))?;

    for (key, value) in systems_data {
        let id = key.parse::<u32>().map_err(|_| format!("System key not an integer: {}", key))?;

        let name = value["name"].as_str().ok_or_else(|| format!("System {} has no name", id))?.to_owned();

        let security = value["security"]
            .as_str().ok_or_else(|| format!("System {} has no security", id))?
            .parse::<f32>().map_err(|_| format!("System {} security is not floating point", id))?;

        let class = value["class"]
            .as_str().map(|v| v.parse::<u16>())
            .map_or(Ok(None), |r| r.map(Some))  // https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/option_result.html
            .map_err(|_| format!("System {} class is not an integer", id))?;

        // Remove weird extra systems
        if name == "No System Name" { continue }
        if name.starts_with("V-") && name.len() == 5 { continue }
        if name.starts_with("AD") && name.len() == 5 { continue }

        systems.push(System { id, name, security, class });
    }

    for (from_system_str, value) in gates_data {
        let from_system = 30000000 + from_system_str.parse::<u32>().map_err(|_| format!("Map from system not an integer: {}", from_system_str))?;

        let to_systems = value.as_object().ok_or_else(|| format!("Map to systems is not an object {:?}", value))?;
        for (to_system_str, _) in to_systems {
            let to_system = 30000000 + to_system_str.parse::<u32>().map_err(|_| format!("Map to system not an integer: {}", to_system_str))?;

            gates.push(Gate { from_system, to_system });
        }
    }

    for (wormhole_type, value) in wormhole_data {
        let jump_mass = u32::try_from(value["jump"].as_u64().ok_or_else(|| format!("Wormhole {} missing jump value", wormhole_type))? / 1000000)
            .map_err(|_| format!("Wormhole {} jump value does not fit in u32", wormhole_type))?;

        wormhole_jump_mass.insert(wormhole_type.to_string(), jump_mass);
    }

    wormhole_jump_mass.insert("SML".to_owned(), 5);
    wormhole_jump_mass.insert("MED".to_owned(), 62);
    wormhole_jump_mass.insert("LRG".to_owned(), 375);

    Ok(StaticData { systems, gates, wormhole_jump_mass })
}