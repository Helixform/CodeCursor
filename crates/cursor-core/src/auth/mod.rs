use node_bridge::prelude::*;
use rand::RngCore;
use wasm_bindgen::prelude::*;

fn random_bytes() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; 32];
    rng.try_fill_bytes(&mut bytes);
    bytes
}
