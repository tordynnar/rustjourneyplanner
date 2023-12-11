use std::fs::File;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::io::prelude::*;
use serde::Deserialize;
use eve_sde::*;

#[derive(Debug, Clone, Deserialize)]
struct NameSDE {
    #[serde(alias = "itemID")]
    id: u32,

    #[serde(alias = "itemName")]
    name: String
}

#[derive(Debug, Clone, Deserialize)]
struct StargateSDE {
    destination: u32
}

#[derive(Debug, Clone, Deserialize)]
struct SystemSDE {
    #[serde(alias = "solarSystemID")]
    id: u32,
    security: f64,
    stargates: HashMap<u32,StargateSDE>,
    #[serde(alias = "wormholeClassID")]
    class: Option<u8>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct RegionSDE {
    #[serde(alias = "regionID")]
    id: u32,
    #[serde(alias = "wormholeClassID")]
    class: Option<u8>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct ConstellationSDE {
    #[serde(alias = "constellationID")]
    id: u32,
    #[serde(alias = "wormholeClassID")]
    class: Option<u8>,
}

enum IndexOrName {
    Index(usize),
    Name(String)
}

fn read_from_zip(archive : &mut zip::ZipArchive<File>, index_or_name: IndexOrName) -> (PathBuf, String) {
    let mut file = match index_or_name {
        IndexOrName::Index(index) => archive.by_index(index).unwrap(),
        IndexOrName::Name(name) => archive.by_name(&name).unwrap()
    };
    
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Failed to read data");
    
    (file.enclosed_name().unwrap().to_owned(), content)
}

fn main() {
    let inputfile = File::open(Path::new("sde.zip")).unwrap();

    let mut archive = zip::ZipArchive::new(inputfile).unwrap();

    let files = (0..archive.len()).into_iter().map(|i| {
        let file = archive.by_index(i).unwrap();
        let path = file.enclosed_name().unwrap().to_owned();
        (i, path)
    }).collect::<Vec::<(usize, PathBuf)>>();

    let regions = files.iter().filter_map(|(i, p)| {
        if p.starts_with(PathBuf::from("sde/fsd/universe/abyssal")) { return None }
        if p.starts_with(PathBuf::from("sde/fsd/universe/void")) { return None }
        if p.starts_with(PathBuf::from("sde/fsd/universe/eve/UUA-F4")) { return None } // Dev Region
        if p.starts_with(PathBuf::from("sde/fsd/universe/eve/A821-A")) { return None } // Dev Region
        if p.starts_with(PathBuf::from("sde/fsd/universe/eve/J7HZ-F")) { return None } // Dev Region

        match p.file_name().unwrap().to_str().unwrap() {
            "region.staticdata" => Some((*i, p.parent().unwrap().to_owned())),
            _ => None
        }
    }).map(|(region_index, region_path)| {
        let constellations = files.iter().filter_map(|(i, p)| {
            if !p.starts_with(&region_path) { return None; }
            match p.file_name().unwrap().to_str().unwrap() {
                "constellation.staticdata" => Some((*i, p.parent().unwrap().to_owned())),
                _ => None
            }
        }).map(|(constellation_index, constellation_path)| {
            let systems_indicies = files.iter().filter_map(|(i, p)| {
                if !p.starts_with(&constellation_path) { return None; }
                match p.file_name().unwrap().to_str().unwrap() {
                    "solarsystem.staticdata" => Some(*i),
                    _ => None
                }
            }).collect::<Vec<usize>>();

            (constellation_index, systems_indicies)
        }).collect::<Vec<(usize,Vec<usize>)>>();

        (region_index, constellations)
    }).collect::<Vec<(usize,Vec<(usize,Vec<usize>)>)>>();

    let (_, names_content) = read_from_zip(&mut archive, IndexOrName::Name("sde/bsd/invNames.yaml".to_owned()));

    let names = serde_yaml::from_str::<Vec<NameSDE>>(&names_content)
        .expect("Failed to parse invNames.yaml")
        .into_iter()
        .map(|n| (n.id, n.name))
        .collect::<HashMap<_,_>>();


    let mut gate_to_system = HashMap::<u32,u32>::new();
    let mut systems = Vec::<System>::new();

    for (region_index, constellations) in regions {
        let (_, region_content) = read_from_zip(&mut archive, IndexOrName::Index(region_index));

        let region = serde_yaml::from_str::<RegionSDE>(&region_content)
            .expect("Failed to parse region data");

        for (constellation_index, system_indicies) in constellations {
            let (_, constellation_content) = read_from_zip(&mut archive, IndexOrName::Index(constellation_index));

            let constellation = serde_yaml::from_str::<ConstellationSDE>(&constellation_content)
                .expect("Failed to parse contellation_content data");

            for system_index in system_indicies {
                let (_, system_content) = read_from_zip(&mut archive, IndexOrName::Index(system_index));
        
                let system = serde_yaml::from_str::<SystemSDE>(&system_content)
                    .expect("Failed to parse system data");
        
                let name = names.get(&system.id)
                    .expect("System id doesn't match a name")
                    .to_owned();
        
                let security = (system.security * 10.0).round() as i8;
        
                let mut neighbours = Vec::<u32>::new();
        
                for (source, stargate) in system.stargates {
                    match gate_to_system.get(&stargate.destination) {
                        None => {
                            gate_to_system.insert(source, system.id);
                        }
                        Some(destination_system) => {
                            neighbours.push(*destination_system);
                            gate_to_system.remove(&stargate.destination);
                        }
                    }
                }
                
                let rawclass = system.class
                    .or(constellation.class)
                    .or(region.class)
                    .or(match name.as_ref() { "Zarzakh" => Some(SystemClass::Zarzakh as u8), _ => None })
                    .expect("No class for system");

                let class = SystemClass::try_from(rawclass).expect("Unexpected class");

                let systemresult = System { id : system.id, name, security, neighbours, class };
                println!("{:?}", systemresult);
                systems.push(systemresult);
            }
        }
    }

    for (gate, system) in &gate_to_system {
        println!("System {} gate {} doesn't have matching gate in another system", system, gate);
    }

    let outputfile = File::create("../../ref/sde.json").unwrap();
    let mut writer = BufWriter::new(outputfile);
    serde_json::to_writer(&mut writer, &systems).unwrap();
    writer.flush().unwrap();
}
