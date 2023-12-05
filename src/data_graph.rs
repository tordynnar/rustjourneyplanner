use std::collections::HashMap;
use chrono::NaiveDateTime;
use petgraph::graph::{Graph, NodeIndex};

use crate::data_dynamic::*;
use crate::data_static::*;

#[derive(Debug, Clone)]
pub struct Wormhole {
    pub signature : Option<String>,
    pub other_signature : Option<String>,
    pub wormhole_type : Option<String>,
    pub lifetime : NaiveDateTime,
    pub life : WormholeLife,
    pub mass : WormholeMass,
    pub jump_mass : Option<u32>
}

pub fn get_graph_data(static_data : StaticData, tripwire_data : Vec::<TripwireWormhole>) -> Result<Graph::<System, Option<Wormhole>>,String> {
    let mut graph = Graph::<System, Option<Wormhole>>::new();
    let mut node_index = HashMap::<u32, NodeIndex>::new();

    for system in static_data.systems {
        node_index.insert(system.id, graph.add_node(system));
    }

    for gate in static_data.gates {
        // Static data gates are already directed, no need to add twice
        graph.add_edge(
            *node_index.get(&gate.from_system).ok_or_else(|| format!("Gate from system {} missing from static data", gate.from_system))?,
            *node_index.get(&gate.to_system).ok_or_else(|| format!("Gate to system {} missing from static data", gate.to_system))?,
            None
        );
    }

    for wormhole in tripwire_data {
        let jump_mass = match wormhole.wormhole_type { None => None, Some(ref v) => static_data.wormhole_jump_mass.get(v).cloned() };

        let from_index = *node_index.get(&wormhole.from_system).ok_or_else(|| format!("Wormhole from system {} missing from static data", wormhole.from_system))?;
        let to_index = *node_index.get(&wormhole.from_system).ok_or_else(|| format!("Wormhole from system {} missing from static data", wormhole.from_system))?;

        graph.add_edge(
            from_index, to_index,
            Some(Wormhole {
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
            Some(Wormhole {
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