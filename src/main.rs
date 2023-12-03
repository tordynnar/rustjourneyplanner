use std::collections::HashMap;
use std::convert::From;
use chrono::NaiveDateTime;
use leptos::*;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct System {
    id : u32,
    name: String,
    security: f32,
    class: Option<u16>
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Gate {
    from_system_id : u32,
    to_system_id: u32
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct StaticData {
    systems : Vec::<System>,
    gates : Vec::<Gate>,
    wormhole_jump_mass : HashMap::<String,u32>
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum SystemOrClass {
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
    Trig,
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
            Some(10) => SystemOrClass::Trig,
            Some(v) => SystemOrClass::SpecificSystem(v)
        }
    }
}

async fn get_static_data() -> Result<StaticData, String> {
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

        systems.push(System { id, name, security, class });
    }

    for (from_system_str, value) in gates_data {
        let from_system_id = 30000000 + from_system_str.parse::<u32>().map_err(|_| format!("Map from system not an integer: {}", from_system_str))?;

        let to_systems = value.as_object().ok_or_else(|| format!("Map to systems is not an object {:?}", value))?;
        for (to_system_str, _) in to_systems {
            let to_system_id = 30000000 + to_system_str.parse::<u32>().map_err(|_| format!("Map to system not an integer: {}", to_system_str))?;

            gates.push(Gate { from_system_id, to_system_id });
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

async fn get_tripwire_data() -> Result<String, String> {
    let baseurl = web_sys::window().ok_or_else(|| format!("Cannot get base URL"))?.origin();

    let client = reqwest::Client::new();
    let result = client.post(format!("{baseurl}/refresh.php"))
        .form(&HashMap::from([
            ("mode", "init"),
            ("systemID", "30000142"),
            ("systemName", "Jita")
        ]))
        .send().await.map_err(|_| format!("Failed to POST refresh.php"))?
        .error_for_status().map_err(|_| format!("Bad status code getting refresh.php"))?
        .bytes().await.map_err(|_| format!("Failed to get bytes for refresh.php"))?;

    let json : serde_json::Value = serde_json::from_slice(&result)
        .map_err(|_| format!("Failed to parse combine.js JSON"))?;

    let wormholes = json["wormholes"].as_object().ok_or_else(|| format!("Wormholes not present in refresh.php"))?;

    for (wormhole_id, wormhole) in wormholes {
        let initial_id = wormhole["initialID"].as_str().ok_or_else(|| format!("initialID missing from wormhole {}", wormhole_id))?;
        let secondary_id = wormhole["secondaryID"].as_str().ok_or_else(|| format!("secondaryID missing from wormhole {}", wormhole_id))?;

        let initial_system_id = match json["signatures"][initial_id]["systemID"].as_str().and_then(|v| v.parse::<u32>().ok()) {
            Some(v) => v,
            None => continue
        };

        let secondary_system_id = SystemOrClass::from(json["signatures"][secondary_id]["systemID"].as_str().and_then(|v| v.parse::<u32>().ok()));

        let initial_signature_id = json["signatures"][initial_id]["signatureID"].as_str().map_or(None, |v| match v { "???" => None, _ => Some(v.to_uppercase()) });
        let secondary_signature_id = json["signatures"][secondary_id]["signatureID"].as_str().map_or(None, |v| match v { "???" => None, _ => Some(v.to_uppercase()) });

        let wormhole_type = wormhole["type"].as_str().map_or(None, |v| match v { "????" => None, "" => None, _ => Some(v) });

        let lifetime_str = json["signatures"][initial_id]["lifeTime"].as_str().ok_or_else(|| format!("lifeTime missing from wormhole {}", wormhole_id))?;
        let lifetime = NaiveDateTime::parse_from_str(lifetime_str, "%Y-%m-%d %H:%M:%S").map_err(|_| format!("lifeTime wrong datetime format for wormhole {}", wormhole_id))?;

        // Probably created by a deathclone
        if initial_signature_id == None && secondary_signature_id == None { continue }

        // Don't want gates, already have then in the static data
        if wormhole_type == Some("GATE") || initial_signature_id == Some("GAT".to_owned()) || secondary_signature_id == Some("GAT".to_owned()) { continue }

        logging::log!("{:?} {:?} {:?} {:?} {:?} {:?}", initial_system_id, secondary_system_id, initial_signature_id, secondary_signature_id, wormhole_type, lifetime);
    }

    Ok(format!("{:?}", json["signatures"]))
}

fn main() {
    mount_to_body(App);
}

#[component]
pub fn App() -> impl IntoView {
    let static_data = create_local_resource(|| (), |_| async move {
        get_static_data().await
    });

    let tripwire_data = create_local_resource(|| (), |_| async move {
        get_tripwire_data().await
    });

    view! {
        {move || match tripwire_data.get() {
            None => view! { <p>"Loading..."</p> }.into_view(),
            Some(Err(err)) => view! { <p>"Error: "{ err }</p> }.into_view(),
            Some(Ok(data)) => view! { <p>{data}</p> }.into_view()
        }}
        <hr/>
        {move || match static_data.get() {
            None => view! { <p>"Loading..."</p> }.into_view(),
            Some(Err(err)) => view! { <p>"Error: "{ err }</p> }.into_view(),
            Some(Ok(data)) => view! {
                <ul>
                    {data.systems.into_iter()
                        .map(|system| view! { <li>{system.name}</li>})
                        .collect_view()}
                </ul>
            }.into_view()
        }}
    }
}
