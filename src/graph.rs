use std::collections::HashMap;
use chrono::NaiveDateTime;
use petgraph::graph::{Graph, NodeIndex};
use eve_sde::System;
use tracing::{info,warn};

use crate::tripwire::{WormholeLife,WormholeMass,TripwireWormhole,SystemOrClass};

#[derive(Debug, Clone)]
pub struct WormholeAttributes {
    pub signature : Option<String>,
    pub other_signature : Option<String>,
    pub wormhole_type : Option<String>,
    pub lifetime : NaiveDateTime,
    pub life : WormholeLife,
    pub mass : WormholeMass,
    pub jump_mass : Option<u32>
}

#[derive(Debug, Clone)]
pub enum Connection {
    Wormhole(WormholeAttributes),
    Gate
}

pub fn get_graph_data(sde : Vec<System>, tripwire_data : Vec::<TripwireWormhole>) -> Result<Graph::<System, Connection>,String> {
    info!("Constructing graph of systems, gates and wormholes");

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

    for wormhole in tripwire_data {
        let jump_mass = None; // TODO

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
                lifetime : wormhole.lifetime.clone(),
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
                lifetime : wormhole.lifetime,
                life : wormhole.life,
                mass : wormhole.mass,
                jump_mass
            })
        );
    }

    Ok(graph)
}