use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::*;

pub struct WebAuthnServer {
    webauthn: Webauthn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredential {
    pub credential_id: String,
    pub credential: Passkey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationChallenge {
    pub challenge: CreationChallengeResponse,
    pub state: PasskeyRegistration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationChallenge {
    pub challenge: RequestChallengeResponse,
    pub state: PasskeyAuthentication,
}

impl WebAuthnServer {
    pub fn new(rp_id: &str, rp_origin: &url::Url, rp_name: &str) -> Result<Self> {
        let builder = WebauthnBuilder::new(rp_id, rp_origin)
            .map_err(|e| Error::WebAuthn(e.to_string()))?
            .rp_name(rp_name);

        let webauthn = builder.build().map_err(|e| Error::WebAuthn(e.to_string()))?;

        Ok(Self { webauthn })
    }

    pub fn start_registration(
        &self,
        user_id: &[u8],
        user_name: &str,
        user_display_name: &str,
        existing_credentials: Option<Vec<CredentialID>>,
    ) -> Result<RegistrationChallenge> {
        let (challenge, state) = self
            .webauthn
            .start_passkey_registration(
                Uuid::from_slice(user_id).unwrap_or_else(|_| Uuid::new_v4()),
                user_name,
                user_display_name,
                existing_credentials,
            )
            .map_err(|e| Error::WebAuthn(e.to_string()))?;

        Ok(RegistrationChallenge { challenge, state })
    }

    pub fn finish_registration(
        &self,
        response: &RegisterPublicKeyCredential,
        state: &PasskeyRegistration,
    ) -> Result<StoredCredential> {
        let credential = self
            .webauthn
            .finish_passkey_registration(response, state)
            .map_err(|e| Error::WebAuthn(e.to_string()))?;

        let credential_id = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            credential.cred_id(),
        );

        Ok(StoredCredential {
            credential_id,
            credential,
        })
    }

    pub fn start_authentication(
        &self,
        credentials: &[Passkey],
    ) -> Result<AuthenticationChallenge> {
        let (challenge, state) = self
            .webauthn
            .start_passkey_authentication(credentials)
            .map_err(|e| Error::WebAuthn(e.to_string()))?;

        Ok(AuthenticationChallenge { challenge, state })
    }

    pub fn finish_authentication(
        &self,
        response: &PublicKeyCredential,
        state: &PasskeyAuthentication,
    ) -> Result<AuthenticationResult> {
        self.webauthn
            .finish_passkey_authentication(response, state)
            .map_err(|e| Error::WebAuthn(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    pub user_id: Vec<u8>,
    pub credentials: Vec<StoredCredential>,
}

impl UserCredentials {
    pub fn new(user_id: Vec<u8>) -> Self {
        Self {
            user_id,
            credentials: Vec::new(),
        }
    }

    pub fn add_credential(&mut self, credential: StoredCredential) {
        self.credentials.push(credential);
    }

    pub fn remove_credential(&mut self, credential_id: &str) -> bool {
        let len_before = self.credentials.len();
        self.credentials.retain(|c| c.credential_id != credential_id);
        self.credentials.len() < len_before
    }

    pub fn passkeys(&self) -> Vec<Passkey> {
        self.credentials.iter().map(|c| c.credential.clone()).collect()
    }

    pub fn credential_ids(&self) -> Vec<CredentialID> {
        self.credentials
            .iter()
            .map(|c| c.credential.cred_id().clone())
            .collect()
    }
}

pub fn generate_user_handle() -> Vec<u8> {
    uuid::Uuid::new_v4().as_bytes().to_vec()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttestationConveyance {
    None,
    Indirect,
    Direct,
    Enterprise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserVerification {
    Required,
    Preferred,
    Discouraged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthenticatorAttachment {
    Platform,
    CrossPlatform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnOptions {
    pub attestation: AttestationConveyance,
    pub user_verification: UserVerification,
    pub authenticator_attachment: Option<AuthenticatorAttachment>,
    pub resident_key: bool,
    pub timeout: u32,
}

impl Default for WebAuthnOptions {
    fn default() -> Self {
        Self {
            attestation: AttestationConveyance::None,
            user_verification: UserVerification::Preferred,
            authenticator_attachment: None,
            resident_key: false,
            timeout: 60000,
        }
    }
}
