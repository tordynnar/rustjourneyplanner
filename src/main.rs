use std::result::Result::{Ok, Err};
use anyhow::{anyhow, Result};
use leptos::*;

#[derive(Debug, Clone)]
struct DatabaseEntry {
    key: String,
    value: i64,
}

async fn get_data() -> Result<String> {
    let baseurl = web_sys::window().ok_or(anyhow!("Cannot get base URL"))?.origin();
    let result = reqwest::get(format!("{baseurl}/js/combine.js")).await?.bytes().await?;
    let json : serde_json::Value = serde_json::from_slice(&result[14..])?;
    let systems = json["systems"].as_object().ok_or_else(|| anyhow!("Systems missing from combine.js"))?;

    for (key, value) in systems {
        let system = key.parse::<u32>().map_err(|_| anyhow!("System key not an integer: {}", key))?;

        let name = value["name"].as_str().ok_or_else(|| anyhow!("System {} has no name", system))?;

        let security = value["security"]
            .as_str().ok_or_else(|| anyhow!("System {} has no security", system))?
            .parse::<f32>().map_err(|_| anyhow!("System {} security is not floating point", system))?;

        let class = value["class"]
            .as_str().map(|v| v.parse::<u16>())
            .map_or(Ok(None), |r| r.map(Some))
            .map_err(|_| anyhow!("System {} class is not an integer", system))?;

        logging::log!("System: {}, Name: {}, Security: {}, Class: {}", system, name, security, class.unwrap_or(0));
    }

    let w = json["systems"]["30000001"]["name"].as_str().ok_or(anyhow!("Not a string"))?;
    Ok(w.to_owned())
}

fn main() {
    mount_to_body(App);
}

#[component]
pub fn App() -> impl IntoView {
    let (data, set_data) = create_signal(vec![
        DatabaseEntry {
            key: "faaoo".to_string(),
            value: 10,
        },
        DatabaseEntry {
            key: "bar".to_string(),
            value: 20,
        },
        DatabaseEntry {
            key: "baz".to_string(),
            value: 15,
        },
    ]);

    let once = create_resource(|| (), |_| async move {
        let result = get_data().await;
        match result {
            Ok(r) => r,
            Err(e) => e.to_string()
        }
    });

    view! {
        <button on:click=move |_| {
            set_data.update(|data| {
                for row in data {
                    row.value *= 2;
                }
            });
            logging::log!("{:?}", data.get());
        }>
            "Update Values"
        </button>
        {move || match once.get() {
            None => view! { <p>"Loading..."</p> }.into_view(),
            Some(data) => view! { <p inner_html=data></p> }.into_view()
        }}
        <For
            each=data
            key=|state| (state.key.clone(), state.value)
            let:child
        >
            <p>{child.value}</p>
        </For>
    }
}






/*
#[component]
fn App() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    
    view! {
        <button
            on:click=move |_| {
                set_count.update(|n| *n += 1);
            }
            style=move || { format!("background-color: {};", if count() % 2 == 0 {"red"} else {"blue"}) }
        >
            "Click me: "
            {count}
        </button>
        <button
            on:click=move |_| {
                set_count.update(|n| *n += 1);
            }
        >
            "Click me: "
            {count}
        </button>
    }
}
*/