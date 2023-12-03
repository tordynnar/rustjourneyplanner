use std::collections::HashMap;
use std::convert::From;
use chrono::{NaiveDateTime, Utc, Duration};
use petgraph::graph::{Graph, NodeIndex};
use leptonic::prelude::*;
use leptos::*;
use web_sys;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct System {
    id : u32,
    name: String,
    security: f32,
    class: Option<u16>
}

impl PartialEq for System {
    fn eq(&self, other: &System) -> bool {
        self.id == other.id
    }
}

impl Eq for System {}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Gate {
    from_system : u32,
    to_system: u32
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum WormholeLife {
    Stable,
    EOL
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum WormholeMass {
    Stable,
    Destab,
    VOC
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
struct TripwireWormhole {
    from_system : u32,
    to_system : SystemOrClass,
    from_signature : Option<String>,
    to_signature : Option<String>,
    wormhole_type : Option<String>,
    lifetime : NaiveDateTime,
    life : WormholeLife,
    mass : WormholeMass
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Wormhole {
    signature : Option<String>,
    other_signature : Option<String>,
    wormhole_type : Option<String>,
    lifetime : NaiveDateTime,
    life : WormholeLife,
    mass : WormholeMass,
    jump_mass : Option<u32>
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

async fn get_tripwire_data() -> Result<Vec::<TripwireWormhole>, String> {
    let mut data = Vec::<TripwireWormhole>::new();

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

        let from_system = match json["signatures"][initial_id]["systemID"].as_str().and_then(|v| v.parse::<u32>().ok()) {
            Some(v) => v,
            None => continue
        };

        let to_system = SystemOrClass::from(json["signatures"][secondary_id]["systemID"].as_str().and_then(|v| v.parse::<u32>().ok()));

        let from_signature = json["signatures"][initial_id]["signatureID"].as_str().and_then(|v| match v { "???" => None, _ => Some(v.to_uppercase()) });
        let to_signature = json["signatures"][secondary_id]["signatureID"].as_str().and_then(|v| match v { "???" => None, _ => Some(v.to_uppercase()) });

        let wormhole_type = wormhole["type"].as_str().and_then(|v| match v { "????" => None, "" => None, _ => Some(v.to_owned()) });

        let lifetime_str = json["signatures"][initial_id]["lifeTime"].as_str().ok_or_else(|| format!("lifeTime missing from wormhole {}", wormhole_id))?;
        let lifetime = NaiveDateTime::parse_from_str(lifetime_str, "%Y-%m-%d %H:%M:%S").map_err(|_| format!("lifeTime wrong datetime format for wormhole {}", wormhole_id))?;
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
            Some(_) => Err(format!("life is not stable or critical for wormhole {}", wormhole_id)),
            None => Err(format!("life missing from wormhole {}", wormhole_id))
        }?;

        let mass = match wormhole["mass"].as_str() {
            Some("stable") => Ok(WormholeMass::Stable),
            Some("destab") => Ok(WormholeMass::Destab),
            Some("critical") => Ok(WormholeMass::VOC),
            Some(_) => Err(format!("mass is not stable, destab or critical for wormhole {}", wormhole_id)),
            None => Err(format!("mass is missing from wormhole {}", wormhole_id))
        }?;

        // Wormholes older than 24 hours probably don't exist any more
        if age > Duration::hours(24) { continue }

        // Probably created by a deathclone
        if from_signature == None && to_signature == None { continue }

        // Don't want gates, already have then in the static data
        if wormhole_type == Some("GATE".to_owned()) || from_signature == Some("GAT".to_owned()) || to_signature == Some("GAT".to_owned()) { continue }

        data.push(TripwireWormhole { from_system, to_system, from_signature, to_signature, wormhole_type, lifetime, life, mass });
    }

    Ok(data)
}

fn get_graph_data(static_data : StaticData, tripwire_data : Vec::<TripwireWormhole>) -> Result<Graph::<System, Option<Wormhole>>,String> {
    let mut graph = Graph::<System, Option<Wormhole>>::new();
    let mut node_index = HashMap::<u32, NodeIndex>::new();

    for system in static_data.systems {
        node_index.insert(system.id, graph.add_node(system));
    }

    for gate in static_data.gates {
        // Static data gates are already directed, no need to add twice
        graph.add_edge(
            *node_index.get(&gate.from_system).ok_or_else(|| format!("Gate from system {} missing from static data", gate.from_system))?,
            *node_index.get(&gate.to_system).ok_or_else(|| format!("Gate to system {} missing from static data", gate.to_system))?,
            None
        );
    }

    for wormhole in tripwire_data {
        let jump_mass = match wormhole.wormhole_type { None => None, Some(ref v) => static_data.wormhole_jump_mass.get(v).cloned() };

        let from_index = *node_index.get(&wormhole.from_system).ok_or_else(|| format!("Wormhole from system {} missing from static data", wormhole.from_system))?;
        let to_index = *node_index.get(&wormhole.from_system).ok_or_else(|| format!("Wormhole from system {} missing from static data", wormhole.from_system))?;

        graph.add_edge(
            from_index, to_index,
            Some(Wormhole {
                signature : wormhole.from_signature.clone(),
                other_signature : wormhole.to_signature.clone(),
                wormhole_type : wormhole.wormhole_type.clone(),
                lifetime : wormhole.lifetime.clone(),
                life : wormhole.life.clone(),
                mass : wormhole.mass.clone(),
                jump_mass : jump_mass.clone()
            })
        );

        graph.add_edge(
            to_index, from_index,
            Some(Wormhole {
                signature : wormhole.to_signature,
                other_signature : wormhole.from_signature,
                wormhole_type : wormhole.wormhole_type,
                lifetime : wormhole.lifetime,
                life : wormhole.life,
                mass : wormhole.mass,
                jump_mass
            })
        );
    }

    Ok(graph)
}

#[component]
pub fn App() -> impl IntoView {
    let static_data = create_local_resource(|| (), |_| async move {
        get_static_data().await
    });

    let tripwire_data = create_local_resource(|| (), |_| async move {
        get_tripwire_data().await
    });

    let graph_data = Signal::derive(move ||  {
        get_graph_data(
            static_data.get().map_or_else(|| Err(format!("Static data loading...")), |v| v)?,
            tripwire_data.get().map_or_else(|| Err(format!("Tripwire data loading...")), |v| v)?
        )
    });

    let systems_data = Signal::derive(move ||  {
        match static_data.get() {
            Some(Ok(v)) => v.systems.iter().map(|v| v.clone()).collect(),
            None | Some(Err(_)) => Vec::<System>::new()
        }
    });

    let (selected_opt, set_selected_opt) = create_signal(Option::<System>::None);

    view! {
        <Root default_theme=LeptonicTheme::default()>
            <ThemeToggle off=LeptonicTheme::Light on=LeptonicTheme::Dark/>
            
            <OptionalSelect
                options=systems_data
                search_text_provider=move |o| format!("{o:?}")
                render_option=move |o| format!("{o:?}")
                selected=move || selected_opt.get()
                set_selected=move |v| set_selected_opt.set(v)
                allow_deselect=true
            />

            {move || match graph_data.get() {
                Err(e) => view! { <p>{ format!("{:?}", e) }</p> }.into_view(),
                Ok(v) => view! { <p>{ format!("{:?}", v) }</p> }.into_view(),
            }}
            {move || match tripwire_data.get() {
                None => view! { <p>"Loading..."</p> }.into_view(),
                Some(Err(err)) => view! { <p>"Error: "{ err }</p> }.into_view(),
                Some(Ok(data)) => view! { <p>{ format!("{:?}", data) }</p> }.into_view()
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
        </Root>
    }
}

fn main() {
    mount_to_body(|| {
        view! { <App/>}
    });
}
