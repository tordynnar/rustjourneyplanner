use leptonic::prelude::*;
use leptos::*;
use petgraph::algo;
use petgraph::visit::IntoNodeReferences;
use itertools::Itertools;
use eve_sde::*;

mod data_dynamic;
mod data_static;
mod data_graph;
mod error;

use data_dynamic::*;
use data_static::*;
use data_graph::*;
use error::*;

#[component]
pub fn App() -> impl IntoView {
    let static_data = create_local_resource(|| (), |_| async {
        get_static_data().await
    });

    let tripwire_data = create_local_resource(|| (), |_| async move {
        get_tripwire_data().await
    });

    let systems_data = Signal::derive(move ||  {
        match static_data.get() {
            Some(Ok(v)) => {
                let mut s : Vec<System> = v.clone();
                s.sort();
                s
            },
            None | Some(Err(_)) => Vec::<System>::new()
        }
    });

    let graph_data = Signal::derive(move ||  {
        get_graph_data(
            static_data.get().map_or_else(|| Err(loadingerror("Loading static data")), |v| v.map_err(|e| criticalerror(e)))?,
            tripwire_data.get().map_or_else(|| Err(loadingerror("Loading wormhole data")), |v| v.map_err(|e| criticalerror(e)))?
        ).map_err(|e| criticalerror(e))
    });

    let (from_system, set_from_system) = create_signal(Option::<System>::None);
    let (to_system, set_to_system) = create_signal(Option::<System>::None);
    let (avoid_systems, set_avoid_systems) = create_signal(Vec::<System>::new());
    
    let route_data = Signal::derive(move || -> Result<Vec<(System,Connection)>,ErrorStatus> {
        let graph = graph_data.get()?;
        let from_system_value = from_system.get().ok_or_else(|| inputerror("From system not selected"))?;
        let to_system_value = to_system.get().ok_or_else(|| inputerror("To system not selected"))?;
        let avoid_systems_value = avoid_systems.get();

        let filtered_graph = graph.filter_map(|_, system| {
            if avoid_systems_value.contains(system) { None } else { Some(system.clone()) }
        }, |_, wormhole| {
            Some(wormhole.clone())
        });

        let (from_system_node, _) = filtered_graph.node_references().find(|(_, system)| {
            system.id == from_system_value.id
        }).ok_or_else(|| routingerror("From system not in graph, likely because it was removed by the filtering rules"))?;

        let (to_system_node, _) = filtered_graph.node_references().find(|(_, system)| {
            system.id == to_system_value.id
        }).ok_or_else(|| routingerror("To system not in graph, likely because it was removed by the filtering rules"))?;

        let (_, path) = algo::astar(
            &filtered_graph,
            from_system_node,
            |n| n == to_system_node,
            |_| 1,
            |_| 0,
        ).ok_or_else(|| routingerror("No path between systems"))?;

        let path_details = path.into_iter().tuple_windows::<(_,_)>().map(|(n1, n2)| {
            let connection = filtered_graph.edges_connecting(n1, n2).exactly_one().map_err(|_| criticalerror("Cannot find edge connecting nodes in graph"))?.weight().clone();
            let node = filtered_graph[n2].clone();
            Ok((node, connection))
        }).collect::<Result<Vec<_>,ErrorStatus>>()?;

        Ok(path_details)
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
                allow_deselect=true
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
                allow_deselect=true
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
                Err(err) => match err.category {
                    ErrorCategory::Loading => view! { <Alert variant=AlertVariant::Info title=move || view! { "Loading" }.into_view() >{err.description}</Alert> }.into_view(),
                    ErrorCategory::Input => view! { <Alert variant=AlertVariant::Warn title=move || view! { "Input error" }.into_view() >{err.description}</Alert> }.into_view(),
                    ErrorCategory::Routing => view! { <Alert variant=AlertVariant::Warn title=move || view! { "Routing problem" }.into_view() >{err.description}</Alert> }.into_view(),
                    ErrorCategory::Critical => view! { <Alert variant=AlertVariant::Danger title=move || view! { "Critical error" }.into_view() >{err.description}</Alert> }.into_view()
                },
                Ok(values) => view! {
                    <table>
                        <thead>
                            <tr>
                                <th>"System"</th>
                                <th>"Connection"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {values.into_iter().map(|(system, connection) : (System, Connection)| {
                                view! {
                                    <tr>
                                        <td>{ format!("{:?}", system) }</td>
                                        <td>{ format!("{:?}", connection) }</td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                }.into_view(),
            }}

        </Root>
    }
}

fn main() {
    mount_to_body(|| {
        view! { <App/>}
    });
}
