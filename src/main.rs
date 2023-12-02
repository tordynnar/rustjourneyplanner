use std::collections::HashMap;
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

async fn get_static_data() -> Result<(Vec::<System>, Vec::<Gate>), String> {
    let mut systems = Vec::<System>::new();
    let mut gates = Vec::<Gate>::new();

    let baseurl = web_sys::window().ok_or_else(|| format!("Cannot get base URL"))?.origin();

    let result = reqwest::get(format!("{baseurl}/js/combine.js")).await
        .map_err(|_| format!("Failed to send request for combine.js"))?
        .error_for_status().map_err(|_| format!("Bad status code getting combine.js"))?
        .bytes().await
        .map_err(|_| format!("Failed to get bytes from combine.js"))?;

    let json : serde_json::Value = serde_json::from_slice(&result[14..])
        .map_err(|_| format!("Failed to parse combine.js JSON"))?;

    let systems_data = json["systems"].as_object().ok_or_else(|| format!("Systems missing from combine.js"))?;
    let gates_data = json["map"]["shortest"].as_object().ok_or_else(|| format!("Map missing from combine.js"))?;

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

    Ok((systems, gates))
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
        .text().await.map_err(|_| format!("Failed to get text for refresh.php"))?;

    Ok(result)
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
            Some(Ok((systems, _))) => view! {
                <ul>
                    {systems.into_iter()
                        .map(|system| view! { <li>{system.name}</li>})
                        .collect_view()}
                </ul>
            }.into_view()
        }}
    }
}
