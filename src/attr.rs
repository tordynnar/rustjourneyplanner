use std::sync::LazyLock;
use std::collections::HashMap;

pub static WORMHOLE_ATTR: LazyLock<HashMap<String, u32>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("A009".to_string(), 5);
    m.insert("A239".to_string(), 375);
    m.insert("A641".to_string(), 1000);
    m.insert("A982".to_string(), 375);
    m.insert("B041".to_string(), 375);
    m.insert("B274".to_string(), 375);
    m.insert("B449".to_string(), 1000);
    m.insert("B520".to_string(), 375);
    m.insert("B735".to_string(), 375);
    m.insert("C008".to_string(), 5);
    m.insert("C125".to_string(), 62);
    m.insert("C140".to_string(), 2000);
    m.insert("C247".to_string(), 375);
    m.insert("C248".to_string(), 2000);
    m.insert("C391".to_string(), 2000);
    m.insert("C414".to_string(), 375);
    m.insert("C729".to_string(), 375);
    m.insert("D364".to_string(), 375);
    m.insert("D382".to_string(), 375);
    m.insert("D792".to_string(), 1000);
    m.insert("D845".to_string(), 375);
    m.insert("E004".to_string(), 5);
    m.insert("E175".to_string(), 375);
    m.insert("E545".to_string(), 375);
    m.insert("E587".to_string(), 1000);
    m.insert("F135".to_string(), 375);
    m.insert("F216".to_string(), 375);
    m.insert("F353".to_string(), 62);
    m.insert("F355".to_string(), 62);
    m.insert("G008".to_string(), 5);
    m.insert("G024".to_string(), 375);
    m.insert("H121".to_string(), 62);
    m.insert("H296".to_string(), 2000);
    m.insert("H900".to_string(), 375);
    m.insert("I182".to_string(), 375);
    m.insert("J244".to_string(), 62);
    m.insert("J377".to_string(), 62000);
    m.insert("K329".to_string(), 2000);
    m.insert("K346".to_string(), 375);
    m.insert("L005".to_string(), 5);
    m.insert("L031".to_string(), 1000);
    m.insert("L477".to_string(), 375);
    m.insert("L614".to_string(), 62);
    m.insert("M001".to_string(), 5);
    m.insert("M164".to_string(), 375);
    m.insert("M267".to_string(), 375);
    m.insert("M555".to_string(), 1000);
    m.insert("M609".to_string(), 62);
    m.insert("N062".to_string(), 375);
    m.insert("N110".to_string(), 62);
    m.insert("N290".to_string(), 2000);
    m.insert("N432".to_string(), 2000);
    m.insert("N766".to_string(), 375);
    m.insert("N770".to_string(), 375);
    m.insert("N944".to_string(), 2000);
    m.insert("N968".to_string(), 375);
    m.insert("O128".to_string(), 375);
    m.insert("O477".to_string(), 375);
    m.insert("O883".to_string(), 62);
    m.insert("P060".to_string(), 62);
    m.insert("Q003".to_string(), 5);
    m.insert("Q063".to_string(), 62);
    m.insert("Q317".to_string(), 62);
    m.insert("R051".to_string(), 1000);
    m.insert("R081".to_string(), 450);
    m.insert("R259".to_string(), 375);
    m.insert("R474".to_string(), 375);
    m.insert("R943".to_string(), 375);
    m.insert("S047".to_string(), 375);
    m.insert("S199".to_string(), 2000);
    m.insert("S804".to_string(), 62);
    m.insert("S877".to_string(), 375);
    m.insert("T405".to_string(), 375);
    m.insert("T458".to_string(), 62);
    m.insert("U210".to_string(), 375);
    m.insert("U319".to_string(), 2000);
    m.insert("U372".to_string(), 375);
    m.insert("U574".to_string(), 375);
    m.insert("V283".to_string(), 1000);
    m.insert("V301".to_string(), 62);
    m.insert("V753".to_string(), 2000);
    m.insert("V898".to_string(), 375);
    m.insert("V911".to_string(), 2000);
    m.insert("V928".to_string(), 375);
    m.insert("W237".to_string(), 2000);
    m.insert("X450".to_string(), 375);
    m.insert("X702".to_string(), 375);
    m.insert("X877".to_string(), 375);
    m.insert("Y683".to_string(), 375);
    m.insert("Y790".to_string(), 62);
    m.insert("Z006".to_string(), 5);
    m.insert("Z060".to_string(), 62);
    m.insert("Z142".to_string(), 2000);
    m.insert("Z457".to_string(), 375);
    m.insert("Z647".to_string(), 62);
    m.insert("Z971".to_string(), 62);
    m
});
