use leptonic::prelude::*;
use leptos::*;
use petgraph::algo;
use petgraph::visit::IntoNodeReferences;

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

    let (from_system, set_from_system) = create_signal(Option::<System>::None);
    let (to_system, set_to_system) = create_signal(Option::<System>::None);
    let (avoid_systems, set_avoid_systems) = create_signal(Vec::<System>::new());

    let route_data = Signal::derive(move || -> Result<String,String> {
        let graph = graph_data.get()?;
        let from_system_value = from_system.get().ok_or_else(|| format!("From system not selected"))?;
        let to_system_value = to_system.get().ok_or_else(|| format!("To system not selected"))?;

        let filtered_graph = graph.filter_map(|_, system| {
            Some(system.clone())
        }, |_, wormhole| {
            Some(wormhole.clone())
        });

        let (from_system_node, _) = filtered_graph.node_references().find(|(_, system)| {
            system.id == from_system_value.id
        }).ok_or_else(|| format!("From system not in graph"))?;

        let (to_system_node, _) = filtered_graph.node_references().find(|(_, system)| {
            system.id == to_system_value.id
        }).ok_or_else(|| format!("To system not in graph"))?;

        let (_, path) = algo::astar(
            &filtered_graph,
            from_system_node,
            |n| n == to_system_node,
            |_| 1,
            |_| 0,
        ).ok_or_else(|| format!("No path between systems"))?;

        let path_systems = path.into_iter().map(|v| {
            filtered_graph[v].clone()
        }).collect::<Vec<_>>();

        Ok(format!("{:?}", path_systems))
    });

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
                selected=move || from_system.get()
                set_selected=move |v| set_from_system.set(v)
                allow_deselect=false
            />

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
                selected=move || to_system.get()
                set_selected=move |v| set_to_system.set(v)
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
                selected=move || avoid_systems.get()
                set_selected=move |v| set_avoid_systems.set(v)
            />

            {move || match route_data.get() {
                Ok(v) => view! { <p>{ v }</p> }.into_view(),
                Err(err) => view! { <p>{ err }</p> }.into_view(),
            }}
        </Root>
    }
}

fn main() {
    mount_to_body(|| {
        view! { <App/>}
    });
}
