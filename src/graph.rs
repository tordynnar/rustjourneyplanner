use std::collections::HashMap;
use std::ops::Deref;
use petgraph::graph::{Graph, NodeIndex};
use eve_sde::System;
use tracing::{info,warn};

use crate::tripwire::*;
use crate::eve_scout::*;
use crate::helpers::*;
use crate::attr::*;

// TODO: Record the source of the wormhole

#[derive(Debug, Clone)]
pub struct WormholeAttributes {
    pub signature : Option<String>,
    pub other_signature : Option<String>,
    pub wormhole_type : Option<String>,
    pub life : WormholeLife,
    pub mass : WormholeMass,
    pub jump_mass : Option<u32>
}

#[derive(Debug, Clone)]
pub enum Connection {
    Wormhole(WormholeAttributes),
    Gate
}

pub fn get_graph(sde : Vec<System>, tripwire_refresh : Option<TripwireRefresh>, eve_scout_refresh : Option<EveScoutRefresh>) -> NeverEq<Graph<System, Connection>> {
    info!("Constructing graph");
    
    let mut graph = Graph::<System, Connection>::new();
    let mut node_index = HashMap::<u32, NodeIndex>::new();

    for system in &sde {
        node_index.insert(system.id, graph.add_node(system.clone()));
    }

    for system in &sde {
        let index1 = *node_index.get(&system.id).unwrap(); // Can't be missing, we just added it
        for neighbour in &system.neighbours {
            let index2 = *node_index.get(neighbour).unwrap(); // Guaranteed consistent by eve_sde crate
            graph.add_edge(index1, index2, Connection::Gate);
            graph.add_edge(index2, index1, Connection::Gate);
        }
    }

    if let Some(tripwire_refresh) = tripwire_refresh {
        for wormhole in tripwire_refresh.wormholes {
            let jump_mass = wormhole.wormhole_type.as_ref().and_then(|t| WORMHOLE_ATTR.deref().get(t).copied());

            let to_system = match wormhole.to_system {
                SystemOrClass::SpecificSystem(v) => v,
                _ => continue
            };

            let [from_index, to_index] = match [wormhole.from_system, to_system].try_map(|s| { node_index.get(&s) }) {
                Some(s) => s,
                None => { warn!("Tripwire has a system not in the SDE"); continue; }
            };

            graph.add_edge(
                *from_index, *to_index,
                Connection::Wormhole(WormholeAttributes {
                    signature : wormhole.from_signature.clone(),
                    other_signature : wormhole.to_signature.clone(),
                    wormhole_type : wormhole.wormhole_type.clone(),
                    life : wormhole.life.clone(),
                    mass : wormhole.mass.clone(),
                    jump_mass : jump_mass.clone()
                })
            );

            graph.add_edge(
                *to_index, *from_index,
                Connection::Wormhole(WormholeAttributes {
                    signature : wormhole.to_signature,
                    other_signature : wormhole.from_signature,
                    wormhole_type : wormhole.wormhole_type,
                    life : wormhole.life,
                    mass : wormhole.mass,
                    jump_mass
                })
            );
        }
    }

    if let Some(eve_scout_refresh) = eve_scout_refresh {
        for wormhole in eve_scout_refresh.wormholes {
            let jump_mass = WORMHOLE_ATTR.get(&wormhole.wh_type).copied();

            let [in_index, out_index] = match [wormhole.in_system_id, wormhole.out_system_id].try_map(|s| { node_index.get(&s) }) {
                Some(s) => s,
                None => { warn!("Eve-Scout has a system not in the SDE"); continue; }
            };

            graph.add_edge(
                *in_index, *out_index,
                Connection::Wormhole(WormholeAttributes {
                    signature : Some(wormhole.in_signature.clone()),
                    other_signature : Some(wormhole.out_signature.clone()),
                    wormhole_type : Some(wormhole.wh_type.clone()),
                    life : WormholeLife::Stable,
                    mass : WormholeMass::Stable,
                    jump_mass : jump_mass.clone()
                })
            );

            graph.add_edge(
                *out_index, *in_index,
                Connection::Wormhole(WormholeAttributes {
                    signature : Some(wormhole.out_signature.clone()),
                    other_signature : Some(wormhole.in_signature.clone()),
                    wormhole_type : Some(wormhole.wh_type.clone()),
                    life : WormholeLife::Stable,
                    mass : WormholeMass::Stable,
                    jump_mass
                })
            );
        }
    }

    NeverEq::<Graph::<System, Connection>> { value : graph }
}