use crate::{Error, Result, User};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlConfig {
    pub entity_id: String,
    pub sso_url: String,
    pub slo_url: Option<String>,
    pub certificate: String,
    pub acs_url: String,
    pub want_assertions_signed: bool,
    pub want_response_signed: bool,
    pub name_id_format: NameIdFormat,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NameIdFormat {
    Unspecified,
    EmailAddress,
    Persistent,
    Transient,
}

impl NameIdFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            NameIdFormat::Unspecified => "urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified",
            NameIdFormat::EmailAddress => "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress",
            NameIdFormat::Persistent => "urn:oasis:names:tc:SAML:2.0:nameid-format:persistent",
            NameIdFormat::Transient => "urn:oasis:names:tc:SAML:2.0:nameid-format:transient",
        }
    }
}

impl Default for NameIdFormat {
    fn default() -> Self {
        Self::EmailAddress
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdpMetadata {
    pub entity_id: String,
    pub sso_url: String,
    pub slo_url: Option<String>,
    pub certificate: String,
    pub name_id_formats: Vec<NameIdFormat>,
}

impl IdpMetadata {
    pub fn from_xml(xml: &str) -> Result<Self> {
        let entity_id = extract_xml_value(xml, "entityID")
            .ok_or_else(|| Error::Config("Missing entityID".into()))?;

        let sso_url = extract_sso_url(xml)
            .ok_or_else(|| Error::Config("Missing SSO URL".into()))?;

        let slo_url = extract_slo_url(xml);

        let certificate = extract_certificate(xml)
            .ok_or_else(|| Error::Config("Missing certificate".into()))?;

        Ok(Self {
            entity_id,
            sso_url,
            slo_url,
            certificate,
            name_id_formats: vec![NameIdFormat::EmailAddress],
        })
    }
}

fn extract_xml_value(xml: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    let start = xml.find(&pattern)? + pattern.len();
    let end = xml[start..].find('"')? + start;
    Some(xml[start..end].to_string())
}

fn extract_sso_url(xml: &str) -> Option<String> {
    if let Some(start) = xml.find("SingleSignOnService") {
        let section = &xml[start..];
        if let Some(end) = section.find("/>") {
            let tag = &section[..end];
            if tag.contains("HTTP-Redirect") || tag.contains("HTTP-POST") {
                return extract_xml_value(tag, "Location");
            }
        }
    }
    None
}

fn extract_slo_url(xml: &str) -> Option<String> {
    if let Some(start) = xml.find("SingleLogoutService") {
        let section = &xml[start..];
        if let Some(end) = section.find("/>") {
            let tag = &section[..end];
            return extract_xml_value(tag, "Location");
        }
    }
    None
}

fn extract_certificate(xml: &str) -> Option<String> {
    let start_tag = "<X509Certificate>";
    let end_tag = "</X509Certificate>";

    let start = xml.find(start_tag)? + start_tag.len();
    let end = xml.find(end_tag)?;

    Some(xml[start..end].replace(['\n', '\r', ' ', '\t'], ""))
}

#[derive(Debug, Clone)]
pub struct AuthnRequest {
    pub id: String,
    pub issue_instant: String,
    pub issuer: String,
    pub acs_url: String,
    pub name_id_format: NameIdFormat,
    pub force_authn: bool,
    pub is_passive: bool,
}

impl AuthnRequest {
    pub fn new(issuer: impl Into<String>, acs_url: impl Into<String>) -> Self {
        Self {
            id: format!("_{}", uuid::Uuid::new_v4()),
            issue_instant: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            issuer: issuer.into(),
            acs_url: acs_url.into(),
            name_id_format: NameIdFormat::EmailAddress,
            force_authn: false,
            is_passive: false,
        }
    }

    pub fn force_authn(mut self) -> Self {
        self.force_authn = true;
        self
    }

    pub fn passive(mut self) -> Self {
        self.is_passive = true;
        self
    }

    pub fn name_id_format(mut self, format: NameIdFormat) -> Self {
        self.name_id_format = format;
        self
    }

    pub fn to_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
                    xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
                    ID="{id}"
                    Version="2.0"
                    IssueInstant="{instant}"
                    AssertionConsumerServiceURL="{acs}"
                    ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
                    ForceAuthn="{force}"
                    IsPassive="{passive}">
    <saml:Issuer>{issuer}</saml:Issuer>
    <samlp:NameIDPolicy Format="{name_id_format}" AllowCreate="true"/>
</samlp:AuthnRequest>"#,
            id = self.id,
            instant = self.issue_instant,
            acs = self.acs_url,
            issuer = self.issuer,
            force = self.force_authn,
            passive = self.is_passive,
            name_id_format = self.name_id_format.as_str(),
        )
    }

    pub fn to_base64(&self) -> String {
        STANDARD.encode(self.to_xml())
    }

    pub fn to_deflated_base64(&self) -> Result<String> {
        use flate2::write::DeflateEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(self.to_xml().as_bytes())
            .map_err(|e| Error::Other(e.to_string()))?;
        let compressed = encoder.finish()
            .map_err(|e| Error::Other(e.to_string()))?;

        Ok(STANDARD.encode(compressed))
    }
}

#[derive(Debug, Clone)]
pub struct SamlResponse {
    pub id: String,
    pub issuer: String,
    pub status_code: String,
    pub name_id: Option<String>,
    pub session_index: Option<String>,
    pub attributes: std::collections::HashMap<String, Vec<String>>,
    pub not_before: Option<chrono::DateTime<chrono::Utc>>,
    pub not_on_or_after: Option<chrono::DateTime<chrono::Utc>>,
}

impl SamlResponse {
    pub fn from_base64(encoded: &str) -> Result<Self> {
        let decoded = STANDARD.decode(encoded)?;
        let xml = String::from_utf8(decoded)
            .map_err(|e| Error::Other(e.to_string()))?;
        Self::from_xml(&xml)
    }

    pub fn from_xml(xml: &str) -> Result<Self> {
        let id = extract_xml_value(xml, "ID")
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let issuer = extract_issuer(xml)
            .ok_or_else(|| Error::AuthFailed("Missing issuer".into()))?;

        let status_code = extract_status_code(xml)
            .ok_or_else(|| Error::AuthFailed("Missing status code".into()))?;

        if !status_code.contains("Success") {
            return Err(Error::AuthFailed(format!("SAML authentication failed: {}", status_code)));
        }

        let name_id = extract_name_id(xml);
        let session_index = extract_session_index(xml);
        let attributes = extract_attributes(xml);

        let not_before = extract_not_before(xml);
        let not_on_or_after = extract_not_on_or_after(xml);

        Ok(Self {
            id,
            issuer,
            status_code,
            name_id,
            session_index,
            attributes,
            not_before,
            not_on_or_after,
        })
    }

    pub fn is_valid(&self) -> bool {
        let now = chrono::Utc::now();

        if let Some(not_before) = self.not_before {
            if now < not_before {
                return false;
            }
        }

        if let Some(not_on_or_after) = self.not_on_or_after {
            if now >= not_on_or_after {
                return false;
            }
        }

        self.status_code.contains("Success")
    }

    pub fn to_user(&self) -> Result<User> {
        let id = self.name_id.clone()
            .ok_or_else(|| Error::AuthFailed("Missing NameID".into()))?;

        let email = self.attributes.get("email")
            .or_else(|| self.attributes.get("http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress"))
            .and_then(|v| v.first().cloned())
            .or_else(|| {
                if id.contains('@') { Some(id.clone()) } else { None }
            });

        let name = self.attributes.get("displayName")
            .or_else(|| self.attributes.get("http://schemas.xmlsoap.org/ws/2005/05/identity/claims/name"))
            .or_else(|| self.attributes.get("cn"))
            .and_then(|v| v.first().cloned());

        Ok(User {
            id,
            email,
            email_verified: Some(true),
            name,
            picture: None,
            provider: Some("saml".into()),
            extra: self.attributes.iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::Array(v.iter().map(|s| serde_json::Value::String(s.clone())).collect())))
                .collect(),
        })
    }
}

fn extract_issuer(xml: &str) -> Option<String> {
    let start_tag = "<saml:Issuer>";
    let end_tag = "</saml:Issuer>";

    if let Some(start) = xml.find(start_tag) {
        let start = start + start_tag.len();
        if let Some(end) = xml[start..].find(end_tag) {
            return Some(xml[start..start + end].trim().to_string());
        }
    }

    let start_tag = "<Issuer>";
    let end_tag = "</Issuer>";

    if let Some(start) = xml.find(start_tag) {
        let start = start + start_tag.len();
        if let Some(end) = xml[start..].find(end_tag) {
            return Some(xml[start..start + end].trim().to_string());
        }
    }

    None
}

fn extract_status_code(xml: &str) -> Option<String> {
    if let Some(start) = xml.find("StatusCode") {
        let section = &xml[start..];
        return extract_xml_value(section, "Value");
    }
    None
}

fn extract_name_id(xml: &str) -> Option<String> {
    for tag in ["<saml:NameID", "<NameID"] {
        if let Some(start) = xml.find(tag) {
            let section = &xml[start..];
            if let Some(end) = section.find("</") {
                let content = &section[..end];
                if let Some(close) = content.find('>') {
                    return Some(content[close + 1..].trim().to_string());
                }
            }
        }
    }
    None
}

fn extract_session_index(xml: &str) -> Option<String> {
    extract_xml_value(xml, "SessionIndex")
}

fn extract_attributes(xml: &str) -> std::collections::HashMap<String, Vec<String>> {
    let mut attrs = std::collections::HashMap::new();

    let mut remaining = xml;
    while let Some(start) = remaining.find("<saml:Attribute ") {
        remaining = &remaining[start..];

        if let Some(end) = remaining.find("</saml:Attribute>") {
            let attr_xml = &remaining[..end];

            if let Some(name) = extract_xml_value(attr_xml, "Name") {
                let mut values = Vec::new();

                let mut value_remaining = attr_xml;
                while let Some(v_start) = value_remaining.find("<saml:AttributeValue") {
                    value_remaining = &value_remaining[v_start..];
                    if let Some(close) = value_remaining.find('>') {
                        let after_open = &value_remaining[close + 1..];
                        if let Some(v_end) = after_open.find("</saml:AttributeValue>") {
                            values.push(after_open[..v_end].trim().to_string());
                        }
                    }
                    if value_remaining.len() > 1 {
                        value_remaining = &value_remaining[1..];
                    } else {
                        break;
                    }
                }

                if !values.is_empty() {
                    attrs.insert(name, values);
                }
            }

            remaining = &remaining[end..];
        } else {
            break;
        }
    }

    attrs
}

fn extract_not_before(xml: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    extract_xml_value(xml, "NotBefore")
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

fn extract_not_on_or_after(xml: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    extract_xml_value(xml, "NotOnOrAfter")
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

pub struct SamlServiceProvider {
    config: SamlConfig,
    idp_metadata: Option<IdpMetadata>,
}

impl SamlServiceProvider {
    pub fn new(config: SamlConfig) -> Self {
        Self {
            config,
            idp_metadata: None,
        }
    }

    pub fn with_idp_metadata(mut self, metadata: IdpMetadata) -> Self {
        self.idp_metadata = Some(metadata);
        self
    }

    pub fn login_url(&self, relay_state: Option<&str>) -> Result<String> {
        let idp = self.idp_metadata.as_ref()
            .ok_or_else(|| Error::Config("IdP metadata not configured".into()))?;

        let request = AuthnRequest::new(&self.config.entity_id, &self.config.acs_url);
        let encoded = request.to_deflated_base64()?;
        let encoded = urlencoding::encode(&encoded);

        let mut url = format!("{}?SAMLRequest={}", idp.sso_url, encoded);

        if let Some(state) = relay_state {
            url.push_str(&format!("&RelayState={}", urlencoding::encode(state)));
        }

        Ok(url)
    }

    pub fn process_response(&self, saml_response: &str) -> Result<SamlResponse> {
        let response = SamlResponse::from_base64(saml_response)?;

        if !response.is_valid() {
            return Err(Error::AuthFailed("SAML response validation failed".into()));
        }

        Ok(response)
    }

    pub fn metadata_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata"
                     entityID="{entity_id}">
    <md:SPSSODescriptor AuthnRequestsSigned="false"
                        WantAssertionsSigned="{want_signed}"
                        protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
        <md:NameIDFormat>{name_id_format}</md:NameIDFormat>
        <md:AssertionConsumerService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
                                     Location="{acs_url}"
                                     index="0"
                                     isDefault="true"/>
    </md:SPSSODescriptor>
</md:EntityDescriptor>"#,
            entity_id = self.config.entity_id,
            want_signed = self.config.want_assertions_signed,
            name_id_format = self.config.name_id_format.as_str(),
            acs_url = self.config.acs_url,
        )
    }
}
