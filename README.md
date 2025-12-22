# Swissknife SDK

A Rust library for integrating SaaS APIs into Rust software. Swissknife provides unified, type-safe clients for popular cloud services.

## Crates

| Crate | Description |
|-------|-------------|
| `swissknife-auth-sdk` | Authentication (OAuth2, JWT, SAML, LDAP, SCIM, WebAuthn, TOTP) |
| `swissknife-communication-sdk` | Messaging (Twilio, SendGrid, FCM, APNs, Slack, Discord, Telegram) |
| `swissknife-social-sdk` | Social media (Instagram, Facebook, TikTok, Twitter/X, LinkedIn) |
| `swissknife-payments-sdk` | Payments (Stripe, PayPal, Square, Braintree, Adyen) |
| `swissknife-crm-sdk` | CRM (Salesforce, HubSpot, Pipedrive, Zoho, Zendesk Sell) |
| `swissknife-hr-sdk` | HR/HRIS (BambooHR, Gusto, Workday, Deel, Personio, HiBob) |
| `swissknife-automation-sdk` | Automation (Zapier, Make, n8n, Pipedream, Tray.io, Workato) |

## Usage

```toml
[dependencies]
swissknife-auth-sdk = "0.1"
swissknife-communication-sdk = "0.1"
swissknife-social-sdk = "0.1"
swissknife-payments-sdk = "0.1"
swissknife-crm-sdk = "0.1"
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
