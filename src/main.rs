#![feature(array_try_map)]

use leptonic::prelude::*;
use leptos::*;
use leptos_icons::{BsIcon,CgIcon};
use petgraph::algo;
use petgraph::visit::IntoNodeReferences;
use itertools::Itertools;
use tracing::info;
use web_sys;
use eve_sde::*;

mod tripwire;
mod graph;
mod error;
mod nevereq;

use tripwire::*;
use graph::*;
use error::*;

pub async fn get_sde() -> Result<Vec<System>, String> {
    info!("Downloading SDE data");

    let baseurl = web_sys::window().ok_or_else(|| format!("Cannot get base URL"))?.origin();

    let result = reqwest::get(format!("{baseurl}/sde.json")).await
        .map_err(|_| format!("Failed to send request for sde.json"))?
        .error_for_status().map_err(|_| format!("Bad status code getting sde.json"))?
        .text().await
        .map_err(|_| format!("Failed to get bytes for sde.json"))?;

    serde_json::from_str::<Vec<System>>(&result)
        .map_err(|e| format!("Failed to parse sde.json JSON: {:?}", e))
}

fn system_search_filter((s, o) : (String, Vec<System>)) -> Vec<System> {
    let lowercased_search = s.to_lowercase();
    o.into_iter()
        .filter(|it| {
            it.name
                .to_lowercase()
                .starts_with(lowercased_search.as_str())
        })
        .take(20)
        .collect::<Vec<System>>()
}

#[component]
pub fn App() -> impl IntoView {
    let sde = create_local_resource(|| (), |_| async {
        get_sde().await
    });

    let tripwire = create_local_resource(|| (), |_| async move {
        get_tripwire().await
    });

    let systems = Signal::derive(move ||  {
        match sde.get() {
            Some(Ok(v)) => {
                let mut s : Vec<System> = v.clone(); s.sort(); s
            },
            None | Some(Err(_)) => Vec::<System>::new()
        }
    });

    let graph = create_memo(move |_|  {
        Ok(get_graph(
            sde.get().map_or_else(|| Err(loadingerror("Loading static data")), |v| v.map_err(|e| criticalerror(e)))?,
            tripwire.get().map_or_else(|| Err(loadingerror("Loading wormhole data")), |v| v.map_err(|e| criticalerror(e)))?
        ))
    });

    let (from_system, set_from_system) = create_signal(Option::<System>::None);
    let (to_system, set_to_system) = create_signal(Option::<System>::None);
    let (avoid_systems, set_avoid_systems) = create_signal(Vec::<System>::new());
    let (exclude_lowsec, set_exclude_lowsec) = create_signal(false);
    let (exclude_nullsec, set_exclude_nullsec) = create_signal(false);
    let (exclude_voc, set_exclude_voc) = create_signal(false);
    let (exclude_eol, set_exclude_eol) = create_signal(false);
    
    let route = Signal::derive(move || -> Result<Vec<(System,Connection)>,ErrorStatus> {
        let graph = graph.get()?.value;
        let from_system_value = from_system.get().ok_or_else(|| inputerror("From system not selected"))?;
        let to_system_value = to_system.get().ok_or_else(|| inputerror("To system not selected"))?;
        let avoid_systems_value = avoid_systems.get();
        let exclude_lowsec_value = exclude_lowsec.get();
        let exclude_nullsec_value = exclude_nullsec.get();
        let exclude_voc_value = exclude_voc.get();
        let exclude_eol_value = exclude_eol.get();

        let filtered_graph = graph.filter_map(|_, system| {
            if avoid_systems_value.contains(system) { return None }
            if exclude_lowsec_value && system.class == SystemClass::Lowsec { return None }
            if exclude_nullsec_value && system.class == SystemClass::Nullsec { return None }
            Some(system.clone())
        }, |_, connection| {
            if let Connection::Wormhole(wormhole) = connection {
                if exclude_voc_value && wormhole.mass == WormholeMass::VOC { return None }
                if exclude_eol_value && wormhole.life == WormholeLife::EOL { return None }
            }
            Some(connection.clone())
        });

        let (from_system_node, _) = filtered_graph.node_references().find(|(_, system)| {
            system.id == from_system_value.id
        }).ok_or_else(|| routingerror("From system not in graph. It was probably removed by the filtering rules."))?;

        let (to_system_node, _) = filtered_graph.node_references().find(|(_, system)| {
            system.id == to_system_value.id
        }).ok_or_else(|| routingerror("To system not in graph. It was probably removed by the filtering rules."))?;

        info!("Calculating shortest path");
        let (_, path) = algo::astar(
            &filtered_graph,
            from_system_node,
            |n| n == to_system_node,
            |_| 1,
            |_| 0,
        ).ok_or_else(|| routingerror("No path between the systems"))?;

        let path_details = path.into_iter().tuple_windows::<(_,_)>().map(|(n1, n2)| {
            let connection = filtered_graph.edges_connecting(n1, n2).exactly_one().map_err(|_| criticalerror("Cannot find edge connecting nodes in graph"))?.weight().clone();
            let node = filtered_graph[n2].clone();
            Ok((node, connection))
        }).collect::<Result<Vec<_>,ErrorStatus>>()?;

        Ok(path_details)
    });

    view! {
        <Root default_theme=LeptonicTheme::default()>
            <AppBar id="app-bar" height=Height::Em(3.5)>
                <div id="app-bar-content">
                    <Stack id="left" orientation=StackOrientation::Horizontal spacing=Size::Zero>
                        <H3 style="margin: 0 0 0 0.5em">
                            "Journey Planner"
                        </H3>
                    </Stack>
                    <Stack id="right" orientation=StackOrientation::Horizontal spacing=Size::Em(1.0)>
                            <LinkExt href="https://github.com/tordynnar/rustjourneyplanner" target=LinkExtTarget::Blank>
                                <Icon id="github-icon" icon=BsIcon::BsGithub aria_label="GitHub icon"/>
                            </LinkExt>
                            <ThemeToggle off=LeptonicTheme::Light on=LeptonicTheme::Dark style="margin-right: 1em"/>
                    </Stack>
                </div>
            </AppBar>

            <div id="container">
                <Grid spacing=Size::Em(0.6)>
                    <Row>
                        <Col md=3>
                            <div style="width: 100%;">
                                <div style="margin-bottom: 5px;">"From System"</div>
                                <OptionalSelect
                                    options=systems
                                    search_text_provider=move |o : System| o.name
                                    search_filter_provider=system_search_filter
                                    render_option=move |o : System| format!("{}", o.name)
                                    selected=move || from_system.get()
                                    set_selected=move |v| set_from_system.set(v)
                                    allow_deselect=true
                                />
                            </div>
                            <div id="swapbutton">
                                <leptonic-link>
                                    <a>
                                        <Icon
                                            on:click=move |_| {
                                                let new_to_system = from_system.get().clone();
                                                let new_from_system = to_system.get().clone();
                                                set_from_system.set(new_from_system);
                                                set_to_system.set(new_to_system);
                                            }
                                            icon=CgIcon::CgSwap style="font-size: 2.5em;"
                                        />
                                    </a>
                                </leptonic-link>
                            </div>
                        </Col>
                        <Col md=3>
                            <div style="width: 100%;">
                                <div style="margin-bottom: 5px;">"To System"</div>
                                <OptionalSelect
                                    options=systems
                                    search_text_provider=move |o : System| o.name
                                    search_filter_provider=system_search_filter
                                    render_option=move |o : System| format!("{}", o.name)
                                    selected=move || to_system.get()
                                    set_selected=move |v| set_to_system.set(v)
                                    allow_deselect=true
                                />
                            </div>
                        </Col>
                    </Row>
                    <Row>
                        <Col md=12>
                            <div style="width: 100%;">
                                <div style="margin-bottom: 5px;">"Avoid Systems"</div>
                                <Multiselect
                                    options=systems
                                    search_text_provider=move |o : System| o.name
                                    search_filter_provider=system_search_filter
                                    render_option=move |o : System| format!("{}", o.name)
                                    selected=move || avoid_systems.get()
                                    set_selected=move |v| set_avoid_systems.set(v)
                                />
                            </div>
                        </Col>
                    </Row>
                    <Row>
                        <Col md=3>
                            <div class="toggle">
                                <Toggle state=exclude_lowsec set_state=set_exclude_lowsec/>
                                <label>"Exclude Lowsec"</label>
                            </div>
                        </Col>
                        <Col md=3>
                            <div class="toggle">
                                <Toggle state=exclude_voc set_state=set_exclude_voc/>
                                <label>"Exclude VOC"</label>
                            </div>
                        </Col>
                    </Row>
                    <Row>
                        <Col md=3>
                            <div class="toggle">
                                <Toggle state=exclude_nullsec set_state=set_exclude_nullsec/>
                                <label>"Exclude Nullsec"</label>
                            </div>
                        </Col>
                        <Col md=3>
                            <div class="toggle">
                                <Toggle state=exclude_eol set_state=set_exclude_eol/>
                                <label>"Exclude EOL"</label>
                            </div>
                        </Col>
                    </Row>
                </Grid>

                {move || match route.get() {
                    Err(err) => match err.category {
                        ErrorCategory::Loading => view! { <Alert variant=AlertVariant::Info title=move || view! { "Loading" }.into_view() >{err.description}</Alert> }.into_view(),
                        ErrorCategory::Input => view! { <Alert variant=AlertVariant::Warn title=move || view! { "Input Error" }.into_view() >{err.description}</Alert> }.into_view(),
                        ErrorCategory::Routing => view! { <Alert variant=AlertVariant::Warn title=move || view! { "Routing Problem" }.into_view() >{err.description}</Alert> }.into_view(),
                        ErrorCategory::Critical => view! { <Alert variant=AlertVariant::Danger title=move || view! { "Critical Error" }.into_view() >{err.description}</Alert> }.into_view()
                    },
                    Ok(values) => view! {
                        <table id="routetable">
                            <thead>
                                <tr>
                                    <th>"System"</th>
                                    <th>"Class"</th>
                                    <th>"Signature"</th>
                                    <th>"Life"</th>
                                    <th>"Mass"</th>
                                    <th>" "</th>
                                </tr>
                            </thead>
                            <tbody>
                                {values.into_iter().map(|(system, connection) : (System, Connection)| {
                                    let avoid_system_clone = system.clone();
                                    view! {
                                        <tr>
                                            <td>{ system.name }</td>
                                            <td>{
                                                let name = match system.class {
                                                    SystemClass::C1 => "C1",
                                                    SystemClass::C2 => "C2",
                                                    SystemClass::C3 => "C3",
                                                    SystemClass::C4 => "C4",
                                                    SystemClass::C5 => "C5",
                                                    SystemClass::C6 => "C6",
                                                    SystemClass::Highsec => "HS",
                                                    SystemClass::Lowsec => "LS",
                                                    SystemClass::Nullsec => "NS",
                                                    SystemClass::Thera => "Thera",
                                                    SystemClass::C13 => "C13",
                                                    SystemClass::DrifterBarbican => "Drifter (Barbican)",
                                                    SystemClass::DrifterConflux => "Drifter (Conflux)",
                                                    SystemClass::DrifterRedoubt => "Drifter (Redoubt)",
                                                    SystemClass::DrifterSentinel => "Drifter (Sentinel)",
                                                    SystemClass::DrifterVidette => "Drifter (Vidette)",
                                                    SystemClass::Pochven => "Pochven",
                                                    SystemClass::Zarzakh => "Zarzakh",
                                                };
                                                match system.class {
                                                    SystemClass::Highsec => view! { <span style="color: green;">"HS"</span> }.into_view(),
                                                    SystemClass::Lowsec => view! { <span style="color: orange;">"LS"</span> }.into_view(),
                                                    _ => view! { <span style="color: red;">{name}</span> }.into_view()
                                                }
                                            }</td>
                                            {
                                            match connection {
                                                Connection::Wormhole(wormhole) => {
                                                    view! {
                                                        <td>{ wormhole.signature.unwrap_or("???".to_owned())[..3].to_owned() }</td>
                                                        <td>{
                                                            match wormhole.life {
                                                                WormholeLife::Stable => view! { <span style="color: green;">"Stable"</span> }.into_view(),
                                                                WormholeLife::EOL => view! { <span style="color: red;">"EOL"</span> }.into_view()
                                                            }
                                                        }</td>
                                                        <td>{
                                                            match wormhole.mass {
                                                                WormholeMass::Stable => view! { <span style="color: green;">"Stable"</span> }.into_view(),
                                                                WormholeMass::Destab => view! { <span style="color: orange;">"Destab"</span> }.into_view(),
                                                                WormholeMass::VOC => view! { <span style="color: red;">"VOC"</span> }.into_view(),
                                                            }
                                                        }</td>
                                                    }
                                                },
                                                Connection::Gate => {
                                                    view! {
                                                        <td>" "</td>
                                                        <td>" "</td>
                                                        <td>" "</td>
                                                    }
                                                }
                                            }.into_view()
                                            }
                                            <td>
                                                <leptonic-link>
                                                    <a on:click=move |_| { 
                                                        let mut new_avoid_systems : Vec<System> = avoid_systems.get().clone();
                                                        new_avoid_systems.push(avoid_system_clone.clone());
                                                        set_avoid_systems.set(new_avoid_systems);
                                                    }>"Avoid"</a>
                                                </leptonic-link>" | "
                                                <LinkExt href={ format!("https://zkillboard.com/system/{}/", system.id) } target=LinkExtTarget::Blank>
                                                    "zKillboard"
                                                </LinkExt>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    }.into_view(),
                }}
            </div>
        </Root>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default_with_config(
        tracing_wasm::WASMLayerConfigBuilder::default()
            .set_max_level(tracing::Level::TRACE)
            .build(),
    );
    mount_to_body(|| {
        view! { <App/>}
    });
}
