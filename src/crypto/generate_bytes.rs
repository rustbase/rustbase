use rand::{thread_rng, Rng};

pub fn generate_random_bytes() -> [u8; 16] {
    thread_rng().gen::<[u8; 16]>()
}