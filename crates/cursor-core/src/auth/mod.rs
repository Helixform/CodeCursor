pub mod session;
pub mod token;

use node_bridge::bindings::AbortSignal;
use rand::RngCore;
use wasm_bindgen::prelude::*;

use self::session::Session;

fn random_bytes() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; 32];
    let _ = rng.try_fill_bytes(&mut bytes);
    bytes
}

#[wasm_bindgen]
pub fn make_session(abort_signal: AbortSignal) -> Session {
    Session::new(abort_signal)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_random_bytes() {
        let bytes = super::random_bytes();
        assert_eq!(bytes.len(), 32);
    }
}
