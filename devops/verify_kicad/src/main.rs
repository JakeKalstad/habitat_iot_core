use kicad::{KiCad, KiCadConnectionConfig, DocumentType, KiCadObjectType};

fn main() {
    let k = KiCad::new(KiCadConnectionConfig {
        client_name: String::from("kicad-iot-tool"),
        ..Default::default()
    })
    .expect("KiCad not running!");

    println!("Connected to KiCad {}", k.get_version().unwrap());

    // Specify the document type (e.g., PCB, Schematic, etc.)

    // Get the list of open documents of the specified type
    let board = k.get_open_board().expect("Board exists");
    println!("{}", board.name());
    let tracks = board.get_tracks().expect("Get tracks");
    for track in tracks {
        let layer = track.layer();
        println!("layer: {:?}\n width: {}\n start:{:?} - end: {:?}\nnet:{}\n locked: {}", layer, track.width(), track.start(), track.end(), track.net(), track.locked());
    }
    let docs = k.get_open_documents(DocumentType::DOCTYPE_PCB).expect("Get PCBs");
    for d in docs {
        println!("{:?}", d)
    }
    let footprints = board.get_items(&[kicad::protos::enums::KiCadObjectType::KOT_LIB_TEXT]).expect("get footprints");
    for footprint in footprints {
        println!("{:?}", footprint);
    }
}
