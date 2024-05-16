use mic2::mic::find_neovi_mics;
use mic2::types::Result;

fn main() -> Result<()>{
    println!("Finding neovi MIC2 devices...");
    let devices = find_neovi_mics()?;

    println!("Found {} device(s)", devices.len());
    for device in devices {
        println!("{device:#?}");
    }

    Ok(())
}
