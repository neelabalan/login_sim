use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;

use std::collections::HashMap;
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


pub fn dump_json<T: Serialize>(data: &T, file_path: &str) -> std::io::Result<()> {
    let json_string = serde_json::to_string(data)?;
    fs::write(file_path, json_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assing_ip() {
        let users = vec![format!("John"), format!("Green")];
        let userbase = assign_ip_address(users, 3);
        assert_eq!(userbase.get("John").unwrap().len() > 1, true);
    }
}
