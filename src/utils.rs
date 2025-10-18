use rand::rngs::StdRng;
use rand::Rng;
use serde::Serialize;

use std::fs;

pub fn get_random_ip(rng: &mut StdRng) -> String {
    // let mut random: Result<_, rand::Error> = SeedableRng::from_rng(rng);
    // let mut rng = rng::thread_rng();
    format!(
        "{}.{}.{}.{}",
        rng.gen::<u8>(),
        rng.gen::<u8>(),
        rng.gen::<u8>(),
        rng.gen::<u8>()
    )
}

pub fn dump_json<T: Serialize>(data: &T, file_path: &str) -> std::io::Result<()> {
    let json_string = serde_json::to_string(data)?;
    fs::write(file_path, json_string)
}
