use rand::RngCore;

fn random_bytes() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; 32];
    let _ = rng.try_fill_bytes(&mut bytes);
    bytes
}
