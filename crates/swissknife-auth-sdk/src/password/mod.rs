use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Argon2id,
    Argon2i,
    Argon2d,
    Bcrypt,
    Scrypt,
}

impl Algorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Algorithm::Argon2id => "argon2id",
            Algorithm::Argon2i => "argon2i",
            Algorithm::Argon2d => "argon2d",
            Algorithm::Bcrypt => "bcrypt",
            Algorithm::Scrypt => "scrypt",
        }
    }
}

pub struct PasswordHasher {
    algorithm: Algorithm,
    argon2_memory_cost: u32,
    argon2_time_cost: u32,
    argon2_parallelism: u32,
    bcrypt_cost: u32,
    scrypt_log_n: u8,
    scrypt_r: u32,
    scrypt_p: u32,
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self {
            algorithm: Algorithm::Argon2id,
            argon2_memory_cost: 65536,
            argon2_time_cost: 3,
            argon2_parallelism: 4,
            bcrypt_cost: 12,
            scrypt_log_n: 15,
            scrypt_r: 8,
            scrypt_p: 1,
        }
    }
}

impl PasswordHasher {
    pub fn new(algorithm: Algorithm) -> Self {
        Self {
            algorithm,
            ..Default::default()
        }
    }

    pub fn argon2id() -> Self {
        Self::new(Algorithm::Argon2id)
    }

    pub fn bcrypt() -> Self {
        Self::new(Algorithm::Bcrypt)
    }

    pub fn scrypt() -> Self {
        Self::new(Algorithm::Scrypt)
    }

    pub fn argon2_memory_cost(mut self, kb: u32) -> Self {
        self.argon2_memory_cost = kb;
        self
    }

    pub fn argon2_time_cost(mut self, iterations: u32) -> Self {
        self.argon2_time_cost = iterations;
        self
    }

    pub fn argon2_parallelism(mut self, lanes: u32) -> Self {
        self.argon2_parallelism = lanes;
        self
    }

    pub fn bcrypt_cost(mut self, cost: u32) -> Self {
        self.bcrypt_cost = cost;
        self
    }

    pub fn scrypt_params(mut self, log_n: u8, r: u32, p: u32) -> Self {
        self.scrypt_log_n = log_n;
        self.scrypt_r = r;
        self.scrypt_p = p;
        self
    }

    pub fn hash(&self, password: &str) -> Result<String> {
        match self.algorithm {
            Algorithm::Argon2id | Algorithm::Argon2i | Algorithm::Argon2d => {
                self.hash_argon2(password)
            }
            Algorithm::Bcrypt => self.hash_bcrypt(password),
            Algorithm::Scrypt => self.hash_scrypt(password),
        }
    }

    fn hash_argon2(&self, password: &str) -> Result<String> {
        use argon2::{
            password_hash::{rand_core::OsRng, PasswordHasher as _, SaltString},
            Argon2, Params, Version,
        };

        let algorithm = match self.algorithm {
            Algorithm::Argon2id => argon2::Algorithm::Argon2id,
            Algorithm::Argon2i => argon2::Algorithm::Argon2i,
            Algorithm::Argon2d => argon2::Algorithm::Argon2d,
            _ => unreachable!(),
        };

        let params = Params::new(
            self.argon2_memory_cost,
            self.argon2_time_cost,
            self.argon2_parallelism,
            None,
        )
        .map_err(|e| Error::PasswordHash(e.to_string()))?;

        let argon2 = Argon2::new(algorithm, Version::V0x13, params);
        let salt = SaltString::generate(&mut OsRng);

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| Error::PasswordHash(e.to_string()))
    }

    fn hash_bcrypt(&self, password: &str) -> Result<String> {
        bcrypt::hash(password, self.bcrypt_cost).map_err(|e| Error::PasswordHash(e.to_string()))
    }

    fn hash_scrypt(&self, password: &str) -> Result<String> {
        use scrypt::{
            password_hash::{rand_core::OsRng, PasswordHasher as _, SaltString},
            Params, Scrypt,
        };

        let params = Params::new(self.scrypt_log_n, self.scrypt_r, self.scrypt_p, 32)
            .map_err(|e| Error::PasswordHash(e.to_string()))?;

        let salt = SaltString::generate(&mut OsRng);
        let hasher = Scrypt;

        hasher
            .hash_password_customized(password.as_bytes(), None, None, params, &salt)
            .map(|h| h.to_string())
            .map_err(|e| Error::PasswordHash(e.to_string()))
    }
}

pub fn verify(password: &str, hash: &str) -> Result<bool> {
    if hash.starts_with("$argon2") {
        verify_argon2(password, hash)
    } else if hash.starts_with("$2") {
        verify_bcrypt(password, hash)
    } else if hash.starts_with("$scrypt") {
        verify_scrypt(password, hash)
    } else {
        Err(Error::PasswordHash("Unknown hash format".into()))
    }
}

fn verify_argon2(password: &str, hash: &str) -> Result<bool> {
    use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};

    let parsed = PasswordHash::new(hash).map_err(|e| Error::PasswordHash(e.to_string()))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

fn verify_bcrypt(password: &str, hash: &str) -> Result<bool> {
    bcrypt::verify(password, hash).map_err(|e| Error::PasswordHash(e.to_string()))
}

fn verify_scrypt(password: &str, hash: &str) -> Result<bool> {
    use password_hash::PasswordVerifier;
    use scrypt::{password_hash::PasswordHash, Scrypt};

    let parsed = PasswordHash::new(hash).map_err(|e| Error::PasswordHash(e.to_string()))?;

    Ok(Scrypt.verify_password(password.as_bytes(), &parsed).is_ok())
}

pub fn needs_rehash(hash: &str, desired: &PasswordHasher) -> bool {
    if hash.starts_with("$argon2") {
        !matches!(
            desired.algorithm,
            Algorithm::Argon2id | Algorithm::Argon2i | Algorithm::Argon2d
        )
    } else if hash.starts_with("$2") {
        desired.algorithm != Algorithm::Bcrypt
    } else if hash.starts_with("$scrypt") {
        desired.algorithm != Algorithm::Scrypt
    } else {
        true
    }
}

#[derive(Debug, Clone)]
pub struct PasswordStrength {
    pub score: u8,
    pub feedback: Vec<String>,
}

pub fn check_strength(password: &str) -> PasswordStrength {
    let mut score: u8 = 0;
    let mut feedback = Vec::new();

    let len = password.len();
    if len >= 8 {
        score += 1;
    } else {
        feedback.push("Password should be at least 8 characters".into());
    }
    if len >= 12 {
        score += 1;
    }
    if len >= 16 {
        score += 1;
    }

    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if has_lower && has_upper {
        score += 1;
    } else {
        feedback.push("Use both uppercase and lowercase letters".into());
    }

    if has_digit {
        score += 1;
    } else {
        feedback.push("Add numbers".into());
    }

    if has_special {
        score += 1;
    } else {
        feedback.push("Add special characters".into());
    }

    let common_passwords = [
        "password",
        "123456",
        "qwerty",
        "admin",
        "letmein",
        "welcome",
    ];
    let lower = password.to_lowercase();
    if common_passwords.iter().any(|p| lower.contains(p)) {
        score = score.saturating_sub(2);
        feedback.push("Avoid common passwords".into());
    }

    PasswordStrength {
        score: score.min(5),
        feedback,
    }
}
