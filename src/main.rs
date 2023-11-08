use leptos::*;
use reqwest::*;

fn main() {
    leptos::mount_to_body(App);
}

#[derive(Debug, Clone)]
struct DatabaseEntry {
    key: String,
    value: i64,
}

async fn get_data() -> Result<String> {
    let baseurl = web_sys::window().unwrap().origin();
    let result = reqwest::get(format!("{baseurl}/js/combine.js")).await?.text().await;
    result
}


#[component]
pub fn App() -> impl IntoView {
    // start with a set of three rows
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
        // when we click, update each row,
        // doubling its value
        <button on:click=move |_| {
            set_data.update(|data| {
                for row in data {
                    row.value *= 2;
                }
            });
            // log the new value of the signal
            logging::log!("{:?}", data.get());
        }>
            "Update Values"
        </button>
        {move || match once.get() {
            None => view! { <p>"Loading..."</p> }.into_view(),
            Some(data) => view! { <p inner_html=data></p> }.into_view()
        }}
        // iterate over the rows and display each value
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