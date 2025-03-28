use rand::Rng;

pub fn generate_otp() -> String {
    let mut rng = rand::rng();
    let otp: String = (0..6)
        .map(|_| rng.random_range(0..=9).to_string())
        .collect();
    otp
}