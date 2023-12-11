use serde_tuple::*;
use serde_repr::*;
use num_enum::TryFromPrimitive;
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize_repr, Deserialize_repr, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Class {
    C1 = 1,
    C2 = 2,
    C3 = 3,
    C4 = 4,
    C5 = 5,
    C6 = 6,
    Highsec = 7,
    Lowsec = 8,
    Nullsec = 9,
    Thera = 12,
    C13 = 13,
    DrifterSentinel = 14,
    DrifterBarbican = 15,
    DrifterVidette = 16,
    DrifterConflux = 17,
    DrifterRedoubt = 18,
    Pochven = 25,
    Zarzakh = 50
}

#[derive(Debug, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct System {
    pub id : u32,
    pub name : String,
    pub security : i8,
    pub class : Class,
    pub neighbours : Vec<u32> // Neighbours are not repeated on both sides
}

impl PartialEq for System {
    fn eq(&self, other: &System) -> bool {
        self.id == other.id
    }
}

impl Eq for System {}

impl PartialOrd for System {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for System {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}