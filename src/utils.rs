use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;

use std::collections::HashMap;
use std::error::Error;
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

pub fn assign_ip_address(users: Vec<String>, max_range: u8) -> HashMap<String, Vec<String>> {
    let mut user_info = HashMap::new();
    let mut rng = StdRng::seed_from_u64(13);
    for user in users {
        user_info.insert(
            user,
            // (0..rand::thread_rng().gen_range(1..max_range + 1))
            (0..rng.gen_range(1..max_range + 1))
                .map(|_| get_random_ip(&mut rng))
                .collect::<Vec<_>>(),
        );
    }
    user_info
}

pub fn load_from_file(file_path: &str) -> Vec<String> {
    return fs::read_to_string(file_path)
        .expect("Failed to read input")
        .split("\n")
        .map(|s| s.to_string()) // Convert &str to String
        .collect();
}

pub fn dump_csv<T: Serialize>(list: &Vec<T>, file_path: &str) -> Result<(), Box<dyn Error>> {
    let mut writer = csv::Writer::from_path(file_path)?;
    for record in list {
        writer.serialize(record)?;
    }
    writer.flush()?;
    Ok(())
}
pub fn dump_json(map: &HashMap<String, Vec<String>>, file_path: &str) -> std::io::Result<()> {
	let json_string = serde_json::to_string(map)?;
	fs::write(file_path, json_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assing_ip() {
		let users = vec![format!("John"), format!("Green")];
		let userbase = assign_ip_address(users, 3);
        assert_eq!(userbase.get("John").unwrap().len()>1, true);
    }

}