use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct PkceVerifier {
    pub verifier: String,
}

#[derive(Debug, Clone)]
pub struct PkceChallenge {
    pub challenge: String,
    pub verifier: PkceVerifier,
}

impl PkceChallenge {
    pub fn new() -> Self {
        let verifier = Self::generate_verifier();
        let challenge = Self::generate_challenge(&verifier);
        Self {
            challenge,
            verifier: PkceVerifier { verifier },
        }
    }

    fn generate_verifier() -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        URL_SAFE_NO_PAD.encode(bytes)
    }

    fn generate_challenge(verifier: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        URL_SAFE_NO_PAD.encode(hash)
    }

    pub fn verifier(&self) -> &str {
        &self.verifier.verifier
    }
}

impl Default for PkceChallenge {
    fn default() -> Self {
        Self::new()
    }
}
