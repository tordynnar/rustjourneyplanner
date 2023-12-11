use std::collections::HashMap;
use chrono::NaiveDateTime;
use petgraph::graph::{Graph, NodeIndex};
use eve_sde::*;

use crate::data_dynamic::*;

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
    let mut graph = Graph::<System, Connection>::new();
    let mut node_index = HashMap::<u32, NodeIndex>::new();

    for system in &sde {
        node_index.insert(system.id, graph.add_node(system.clone()));
    }

    for system in &sde {
        let index1 = *node_index.get(&system.id).ok_or_else(|| format!("Gate from system {} missing from static data", system.id))?;
        for neighbour in &system.neighbours {
            let index2 = *node_index.get(neighbour).ok_or_else(|| format!("Gate to system {} missing from static data", neighbour))?;
            graph.add_edge(index1, index2, Connection::Gate);
            graph.add_edge(index2, index1, Connection::Gate);
        }
    }

    for wormhole in tripwire_data {
        //let jump_mass = match wormhole.wormhole_type { None => None, Some(ref v) => static_data.wormhole_jump_mass.get(v).cloned() };
        let jump_mass = None;

        let to_system = match wormhole.to_system {
            SystemOrClass::SpecificSystem(v) => v,
            _ => continue
        };

        let from_index = *node_index.get(&wormhole.from_system).ok_or_else(|| format!("Wormhole from system {} missing from static data", wormhole.from_system))?;
        let to_index = *node_index.get(&to_system).ok_or_else(|| format!("Wormhole from system {} missing from static data", to_system))?;

        graph.add_edge(
            from_index, to_index,
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
            to_index, from_index,
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