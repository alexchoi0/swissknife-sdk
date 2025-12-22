# Swissknife SDK

A go-to Rust library for developing software that runs on cloud infrastructure.

## Crates

| Crate | Description |
|-------|-------------|
| `swissknife-auth-sdk` | Authentication (OAuth2, JWT, SAML, LDAP, SCIM, WebAuthn, TOTP) |
| `swissknife-communication-sdk` | Messaging (Twilio, SendGrid, FCM, APNs, Slack, Discord, Telegram) |
| `swissknife-social-sdk` | Social media (Instagram, Facebook, TikTok, Twitter/X, LinkedIn) |

## Usage

```toml
[dependencies]
swissknife-auth-sdk = { git = "https://github.com/alexchoi0/swissknife-sdk" }
swissknife-communication-sdk = { git = "https://github.com/alexchoi0/swissknife-sdk" }
swissknife-social-sdk = { git = "https://github.com/alexchoi0/swissknife-sdk" }
```

## License

MIT
