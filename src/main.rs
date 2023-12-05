use leptonic::prelude::*;
use leptos::*;

mod data_dynamic;
mod data_static;
mod data_graph;

use data_dynamic::*;
use data_static::*;
use data_graph::*;

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
            Some(Ok(v)) => {
                let mut s : Vec<System> = v.systems.iter().map(|v| v.clone()).collect();
                s.sort_by(|s1, s2| s1.name.cmp(&s2.name));
                s
            },
            None | Some(Err(_)) => Vec::<System>::new()
        }
    });

    let (selected_opt, set_selected_opt) = create_signal(Option::<System>::None);

    let (all_selected, set_all_selected) = create_signal(Vec::<System>::new());

    view! {
        <Root default_theme=LeptonicTheme::default()>
            <ThemeToggle off=LeptonicTheme::Light on=LeptonicTheme::Dark/>
            
            <OptionalSelect
                options=systems_data
                search_text_provider=move |o : System| o.name
                search_filter_provider=move |(s, o) : (String, Vec<System>)| {
                    let lowercased_search = s.to_lowercase();
                    o.into_iter()
                        .filter(|it| {
                            it.name
                                .to_lowercase()
                                .starts_with(lowercased_search.as_str())
                        })
                        .take(20)
                        .collect::<Vec<_>>()
                }
                render_option=move |o : System| format!("{}", o.name)
                selected=move || selected_opt.get()
                set_selected=move |v| set_selected_opt.set(v)
                allow_deselect=false
            />

            <Multiselect
                options=systems_data
                search_text_provider=move |o : System| o.name
                search_filter_provider=move |(s, o) : (String, Vec<System>)| {
                    let lowercased_search = s.to_lowercase();
                    o.into_iter()
                        .filter(|it| {
                            it.name
                                .to_lowercase()
                                .starts_with(lowercased_search.as_str())
                        })
                        .take(20)
                        .collect::<Vec<_>>()
                }
                render_option=move |o : System| format!("{}", o.name)
                selected=move || all_selected.get()
                set_selected=move |v| set_all_selected.set(v)
            />

            /*
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
            */
        </Root>
    }
}

fn main() {
    mount_to_body(|| {
        view! { <App/>}
    });
}
