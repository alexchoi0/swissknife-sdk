use crate::{Error, Result};
use totp_rs::{Algorithm, Secret, TOTP};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TotpAlgorithm {
    Sha1,
    Sha256,
    Sha512,
}

impl From<TotpAlgorithm> for Algorithm {
    fn from(alg: TotpAlgorithm) -> Self {
        match alg {
            TotpAlgorithm::Sha1 => Algorithm::SHA1,
            TotpAlgorithm::Sha256 => Algorithm::SHA256,
            TotpAlgorithm::Sha512 => Algorithm::SHA512,
        }
    }
}

pub struct TotpConfig {
    pub algorithm: TotpAlgorithm,
    pub digits: usize,
    pub step: u64,
    pub issuer: String,
    pub account_name: String,
}

impl TotpConfig {
    pub fn new(issuer: impl Into<String>, account_name: impl Into<String>) -> Self {
        Self {
            algorithm: TotpAlgorithm::Sha1,
            digits: 6,
            step: 30,
            issuer: issuer.into(),
            account_name: account_name.into(),
        }
    }

    pub fn algorithm(mut self, algorithm: TotpAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    pub fn digits(mut self, digits: usize) -> Self {
        self.digits = digits;
        self
    }

    pub fn step(mut self, step: u64) -> Self {
        self.step = step;
        self
    }
}

pub struct TotpSecret {
    secret: Secret,
    totp: TOTP,
}

impl TotpSecret {
    pub fn generate(config: &TotpConfig) -> Result<Self> {
        let secret = Secret::generate_secret();

        let totp = TOTP::new(
            config.algorithm.into(),
            config.digits,
            1,
            config.step,
            secret.to_bytes().map_err(|e| Error::Totp(e.to_string()))?,
            Some(config.issuer.clone()),
            config.account_name.clone(),
        )
        .map_err(|e| Error::Totp(e.to_string()))?;

        Ok(Self { secret, totp })
    }

    pub fn from_base32(
        secret_base32: &str,
        config: &TotpConfig,
    ) -> Result<Self> {
        let secret = Secret::Encoded(secret_base32.to_string());

        let totp = TOTP::new(
            config.algorithm.into(),
            config.digits,
            1,
            config.step,
            secret.to_bytes().map_err(|e| Error::Totp(e.to_string()))?,
            Some(config.issuer.clone()),
            config.account_name.clone(),
        )
        .map_err(|e| Error::Totp(e.to_string()))?;

        Ok(Self { secret, totp })
    }

    pub fn secret_base32(&self) -> String {
        self.secret.to_encoded().to_string()
    }

    pub fn generate_code(&self) -> Result<String> {
        self.totp
            .generate_current()
            .map_err(|e| Error::Totp(e.to_string()))
    }

    pub fn generate_code_at(&self, time: u64) -> String {
        self.totp.generate(time)
    }

    pub fn verify(&self, code: &str) -> Result<bool> {
        self.totp
            .check_current(code)
            .map_err(|e| Error::Totp(e.to_string()))
    }

    pub fn verify_with_window(&self, code: &str, window: u64) -> Result<bool> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| Error::Totp(e.to_string()))?
            .as_secs();

        for i in 0..=window {
            let time_before = current_time.saturating_sub(i * self.totp.step);
            let time_after = current_time + i * self.totp.step;

            if self.totp.generate(time_before) == code {
                return Ok(true);
            }
            if i > 0 && self.totp.generate(time_after) == code {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn otpauth_uri(&self) -> String {
        self.totp.get_url()
    }

    pub fn qr_code_base64(&self) -> Result<String> {
        self.totp
            .get_qr_base64()
            .map_err(|e| Error::Totp(e.to_string()))
    }

    pub fn qr_code_png(&self) -> Result<Vec<u8>> {
        self.totp
            .get_qr_png()
            .map_err(|e| Error::Totp(e.to_string()))
    }

    pub fn time_remaining(&self) -> Result<u64> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| Error::Totp(e.to_string()))?
            .as_secs();

        Ok(self.totp.step - (current_time % self.totp.step))
    }
}

pub struct BackupCodes {
    codes: Vec<String>,
    used: Vec<bool>,
}

impl BackupCodes {
    pub fn generate(count: usize) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let codes: Vec<String> = (0..count)
            .map(|_| {
                let code: u32 = rng.gen_range(10000000..99999999);
                format!("{:08}", code)
            })
            .collect();

        let used = vec![false; count];

        Self { codes, used }
    }

    pub fn codes(&self) -> Vec<&str> {
        self.codes.iter().map(|s| s.as_str()).collect()
    }

    pub fn available_codes(&self) -> Vec<&str> {
        self.codes
            .iter()
            .zip(&self.used)
            .filter_map(|(code, &used)| if !used { Some(code.as_str()) } else { None })
            .collect()
    }

    pub fn verify(&mut self, code: &str) -> bool {
        let normalized = code.replace("-", "").replace(" ", "");

        for (i, c) in self.codes.iter().enumerate() {
            if !self.used[i] && *c == normalized {
                self.used[i] = true;
                return true;
            }
        }

        false
    }

    pub fn remaining_count(&self) -> usize {
        self.used.iter().filter(|&&u| !u).count()
    }

    pub fn to_hashed(&self) -> Result<Vec<String>> {
        use crate::password::PasswordHasher;
        let hasher = PasswordHasher::argon2id();

        self.codes
            .iter()
            .map(|code| hasher.hash(code))
            .collect()
    }
}

pub fn generate_recovery_phrase(word_count: usize) -> Vec<String> {
    use rand::seq::SliceRandom;

    const WORDS: &[&str] = &[
        "apple", "banana", "cherry", "delta", "eagle", "forest", "guitar", "harbor",
        "island", "jungle", "kernel", "lemon", "mango", "needle", "orange", "pepper",
        "quartz", "river", "sunset", "tiger", "ultra", "violet", "window", "xray",
        "yellow", "zebra", "anchor", "bridge", "castle", "dragon", "engine", "falcon",
    ];

    let mut rng = rand::thread_rng();
    WORDS
        .choose_multiple(&mut rng, word_count)
        .map(|s| s.to_string())
        .collect()
}
