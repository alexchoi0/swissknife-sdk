use crate::{Result, User};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapConfig {
    pub url: String,
    pub base_dn: String,
    pub bind_dn: Option<String>,
    pub bind_password: Option<String>,
    pub user_search_base: Option<String>,
    pub user_search_filter: String,
    pub group_search_base: Option<String>,
    pub group_search_filter: Option<String>,
    pub user_attributes: UserAttributeMapping,
    pub use_tls: bool,
    pub start_tls: bool,
    pub timeout_secs: u64,
}

impl Default for LdapConfig {
    fn default() -> Self {
        Self {
            url: "ldap://localhost:389".into(),
            base_dn: "dc=example,dc=com".into(),
            bind_dn: None,
            bind_password: None,
            user_search_base: None,
            user_search_filter: "(uid={username})".into(),
            group_search_base: None,
            group_search_filter: Some("(member={user_dn})".into()),
            user_attributes: UserAttributeMapping::default(),
            use_tls: false,
            start_tls: false,
            timeout_secs: 10,
        }
    }
}

impl LdapConfig {
    pub fn new(url: impl Into<String>, base_dn: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            base_dn: base_dn.into(),
            ..Default::default()
        }
    }

    pub fn with_bind_credentials(mut self, bind_dn: impl Into<String>, password: impl Into<String>) -> Self {
        self.bind_dn = Some(bind_dn.into());
        self.bind_password = Some(password.into());
        self
    }

    pub fn with_user_search_filter(mut self, filter: impl Into<String>) -> Self {
        self.user_search_filter = filter.into();
        self
    }

    pub fn with_tls(mut self) -> Self {
        self.use_tls = true;
        self
    }

    pub fn with_start_tls(mut self) -> Self {
        self.start_tls = true;
        self
    }

    pub fn user_search_dn(&self) -> String {
        self.user_search_base.clone().unwrap_or_else(|| self.base_dn.clone())
    }

    pub fn group_search_dn(&self) -> String {
        self.group_search_base.clone().unwrap_or_else(|| self.base_dn.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAttributeMapping {
    pub uid: String,
    pub email: String,
    pub display_name: String,
    pub first_name: String,
    pub last_name: String,
    pub member_of: String,
}

impl Default for UserAttributeMapping {
    fn default() -> Self {
        Self {
            uid: "uid".into(),
            email: "mail".into(),
            display_name: "displayName".into(),
            first_name: "givenName".into(),
            last_name: "sn".into(),
            member_of: "memberOf".into(),
        }
    }
}

impl UserAttributeMapping {
    pub fn active_directory() -> Self {
        Self {
            uid: "sAMAccountName".into(),
            email: "mail".into(),
            display_name: "displayName".into(),
            first_name: "givenName".into(),
            last_name: "sn".into(),
            member_of: "memberOf".into(),
        }
    }

    pub fn openldap() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub struct LdapUser {
    pub dn: String,
    pub uid: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub groups: Vec<String>,
    pub attributes: HashMap<String, Vec<String>>,
}

impl LdapUser {
    pub fn to_user(&self) -> User {
        let name = self.display_name.clone().or_else(|| {
            match (&self.first_name, &self.last_name) {
                (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
                (Some(first), None) => Some(first.clone()),
                (None, Some(last)) => Some(last.clone()),
                (None, None) => None,
            }
        });

        User {
            id: self.uid.clone(),
            email: self.email.clone(),
            email_verified: self.email.as_ref().map(|_| true),
            name,
            picture: None,
            provider: Some("ldap".into()),
            extra: {
                let mut extra = HashMap::new();
                extra.insert("dn".into(), serde_json::Value::String(self.dn.clone()));
                if !self.groups.is_empty() {
                    extra.insert("groups".into(), serde_json::Value::Array(
                        self.groups.iter().map(|g| serde_json::Value::String(g.clone())).collect()
                    ));
                }
                extra
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct LdapGroup {
    pub dn: String,
    pub name: String,
    pub description: Option<String>,
    pub members: Vec<String>,
}

#[async_trait]
pub trait LdapClient: Send + Sync {
    async fn bind(&self, dn: &str, password: &str) -> Result<()>;
    async fn search_user(&self, username: &str) -> Result<Option<LdapUser>>;
    async fn search_groups(&self, user_dn: &str) -> Result<Vec<LdapGroup>>;
    async fn authenticate(&self, username: &str, password: &str) -> Result<LdapUser>;
    async fn test_connection(&self) -> Result<()>;
}

pub struct LdapAuthenticator {
    config: LdapConfig,
}

impl LdapAuthenticator {
    pub fn new(config: LdapConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &LdapConfig {
        &self.config
    }

    pub fn build_user_search_filter(&self, username: &str) -> String {
        self.config.user_search_filter.replace("{username}", username)
    }

    pub fn build_group_search_filter(&self, user_dn: &str) -> Option<String> {
        self.config.group_search_filter.as_ref().map(|f| {
            f.replace("{user_dn}", user_dn)
        })
    }

    pub fn parse_group_cn(&self, group_dn: &str) -> Option<String> {
        group_dn.split(',')
            .next()
            .and_then(|first| first.strip_prefix("cn="))
            .or_else(|| group_dn.split(',').next().and_then(|first| first.strip_prefix("CN=")))
            .map(String::from)
    }
}

#[derive(Debug, Clone)]
pub struct ActiveDirectoryConfig {
    pub domain: String,
    pub server: String,
    pub port: u16,
    pub use_ssl: bool,
    pub base_dn: String,
    pub service_account: Option<String>,
    pub service_password: Option<String>,
}

impl ActiveDirectoryConfig {
    pub fn new(domain: impl Into<String>, server: impl Into<String>) -> Self {
        let domain = domain.into();
        let base_dn = domain.split('.')
            .map(|part| format!("dc={}", part))
            .collect::<Vec<_>>()
            .join(",");

        Self {
            domain,
            server: server.into(),
            port: 389,
            use_ssl: false,
            base_dn,
            service_account: None,
            service_password: None,
        }
    }

    pub fn with_ssl(mut self) -> Self {
        self.use_ssl = true;
        self.port = 636;
        self
    }

    pub fn with_service_account(mut self, account: impl Into<String>, password: impl Into<String>) -> Self {
        self.service_account = Some(account.into());
        self.service_password = Some(password.into());
        self
    }

    pub fn to_ldap_config(&self) -> LdapConfig {
        let protocol = if self.use_ssl { "ldaps" } else { "ldap" };
        let url = format!("{}://{}:{}", protocol, self.server, self.port);

        let mut config = LdapConfig::new(url, &self.base_dn)
            .with_user_search_filter("(sAMAccountName={username})");

        config.user_attributes = UserAttributeMapping::active_directory();
        config.use_tls = self.use_ssl;

        if let (Some(account), Some(password)) = (&self.service_account, &self.service_password) {
            config = config.with_bind_credentials(account, password);
        }

        config
    }

    pub fn user_principal_name(&self, username: &str) -> String {
        if username.contains('@') {
            username.to_string()
        } else {
            format!("{}@{}", username, self.domain)
        }
    }
}

pub fn escape_ldap_filter(value: &str) -> String {
    value
        .replace('\\', "\\5c")
        .replace('*', "\\2a")
        .replace('(', "\\28")
        .replace(')', "\\29")
        .replace('\0', "\\00")
}

pub fn escape_ldap_dn(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(',', "\\,")
        .replace('+', "\\+")
        .replace('"', "\\\"")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace(';', "\\;")
        .replace('=', "\\=")
        .replace('#', "\\#")
}
