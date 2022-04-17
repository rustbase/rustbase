use rand::{thread_rng, Rng};

pub fn generate_random_bytes() -> [u8; 16] {
    let random_bytes = thread_rng().gen::<[u8; 16]>();
    return random_bytes;
}