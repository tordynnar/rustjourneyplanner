use std::result::Result::{Ok, Err};
use std::collections::HashMap;
use anyhow::{anyhow, Result};
use leptos::*;
use petgraph::Graph;
use petgraph::graph::NodeIndex;

#[derive(Debug)]
struct System {
    id : u32,
    name: String,
    security: f32,
    class: Option<u16>
}

#[derive(Debug)]
struct Connection {
    name: String
}

async fn get_data() -> Result<String> {
    let mut map = Graph::<System, Connection>::new();
    let mut system_index = HashMap::<u32, NodeIndex>::new();

    let baseurl = web_sys::window().ok_or(anyhow!("Cannot get base URL"))?.origin();

    let result = reqwest::get(format!("{baseurl}/js/combine.js")).await?.bytes().await?;
    let json : serde_json::Value = serde_json::from_slice(&result[14..])?;

    let systems = json["systems"].as_object().ok_or_else(|| anyhow!("Systems missing from combine.js"))?;

    for (key, value) in systems {
        let id = key.parse::<u32>().map_err(|_| anyhow!("System key not an integer: {}", key))?;

        let name = value["name"].as_str().ok_or_else(|| anyhow!("System {} has no name", id))?.to_owned();

        let security = value["security"]
            .as_str().ok_or_else(|| anyhow!("System {} has no security", id))?
            .parse::<f32>().map_err(|_| anyhow!("System {} security is not floating point", id))?;

        let class = value["class"]
            .as_str().map(|v| v.parse::<u16>())
            .map_or(Ok(None), |r| r.map(Some))  // https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/option_result.html
            .map_err(|_| anyhow!("System {} class is not an integer", id))?;

        system_index.insert(id, map.add_node(System { id, name, security, class }));
    }

    let connections = json["map"]["shortest"].as_object().ok_or_else(|| anyhow!("Map missing from combine.js"))?;

    for (from_system_str, value) in connections {
        let from_system = from_system_str.parse::<u32>().map_err(|_| anyhow!("Map from system not an integer: {}", from_system_str))? + 30000000;
        let from_system_index = system_index.get(&from_system).ok_or_else(|| anyhow!("Map contains non-existant from system: {}", from_system))?;

        let to_systems = value.as_object().ok_or_else(|| anyhow!("Map to systems is not an object {:?}", value))?;
        for (to_system_str, _) in to_systems {
            let to_system = to_system_str.parse::<u32>().map_err(|_| anyhow!("Map to system not an integer: {}", to_system_str))? + 30000000;
            let to_system_index = system_index.get(&to_system).ok_or_else(|| anyhow!("Map contains non-existant to system: {}", to_system))?;

            map.add_edge(*from_system_index, *to_system_index, Connection { name: "test".to_owned() });
        }
    }

    logging::log!("{:?}", map);

    let w = json["systems"]["30000001"]["name"].as_str().ok_or(anyhow!("Not a string"))?;
    Ok(w.to_owned())
}

fn main() {
    mount_to_body(App);
}

#[component]
pub fn App() -> impl IntoView {
    let once = create_resource(|| (), |_| async move {
        let result = get_data().await;
        match result {
            Ok(r) => r,
            Err(e) => e.to_string()
        }
    });

    view! {
        {move || match once.get() {
            None => view! { <p>"Loading..."</p> }.into_view(),
            Some(data) => view! { <p inner_html=data></p> }.into_view()
        }}
    }
}
