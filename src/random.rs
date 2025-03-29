use rand::Rng;
use sha2::Digest;

pub fn generate_otp() -> String {
    let mut rng = rand::rng();
    let otp: String = (0..6)
        .map(|_| rng.random_range(0..=9).to_string())
        .collect();
    otp
}

pub fn random_word(len: usize) -> String {
    let mut rng = rand::rng();
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
    (0..len).map(|_| chars[rng.random_range(0..chars.len())]).collect()
}

pub fn random_hash(value: &str) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(random_word(16).as_bytes());
    hasher.update(value.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}
