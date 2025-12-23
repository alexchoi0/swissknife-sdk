# Swissknife SDK

A comprehensive Rust library for integrating SaaS APIs into Rust software. Swissknife provides unified, type-safe clients for popular cloud services and AI/LLM providers.

## Features

- **Type-safe API clients** with full Rust type coverage
- **Async/await** support using Tokio
- **MCP (Model Context Protocol)** server for AI agent tool integration
- **Feature flags** for minimal binary size - only include what you need
- **Unified error handling** across all providers

## Crates

### AI & LLM

| Crate | Description |
|-------|-------------|
| `swissknife-ai-sdk` | LLM providers (OpenAI, Anthropic, Mistral, DeepL, Stability) with MCP server |
| `swissknife-search-sdk` | Search APIs (Tavily, Exa, Serper, Perplexity, DuckDuckGo, Wikipedia) |
| `swissknife-memory-sdk` | Long-term memory (Mem0, Zep) |
| `swissknife-scraping-sdk` | Web scraping (Apify, Firecrawl, BrowserUse) |
| `swissknife-vectordb-sdk` | Vector databases (Pinecone, Qdrant, Weaviate, ChromaDB, Upstash Vector) |

### Communication & Social

| Crate | Description |
|-------|-------------|
| `swissknife-communication-sdk` | Messaging (Twilio, SendGrid, Slack, Discord, Telegram, Resend, Mailgun, Teams) |
| `swissknife-social-sdk` | Social media (Instagram, Facebook, TikTok, Twitter/X, LinkedIn, Reddit, YouTube, Spotify) |

### Business & Productivity

| Crate | Description |
|-------|-------------|
| `swissknife-productivity-sdk` | Productivity (Notion, Airtable, Google Workspace, Microsoft 365, Confluence, Calendly) |
| `swissknife-crm-sdk` | CRM (Salesforce, HubSpot, Pipedrive, Zoho, Zendesk Sell, Apollo, Clay, Hunter) |
| `swissknife-pm-sdk` | Project management (Jira, Linear, Asana, Monday, ClickUp) |
| `swissknife-hr-sdk` | HR/HRIS (BambooHR, Gusto, Workday, Deel, Personio, HiBob, Rippling) |
| `swissknife-automation-sdk` | Automation (Zapier, Make, n8n, Pipedream, Tray.io, Workato) |

### Finance & Payments

| Crate | Description |
|-------|-------------|
| `swissknife-payments-sdk` | Payments (Stripe, PayPal, Square, Braintree, Adyen) |
| `swissknife-banking-sdk` | Banking APIs (Plaid, MX, Teller, TrueLayer, GoCardless, Yapily) |
| `swissknife-markets-sdk` | Prediction markets (Kalshi, Polymarket) |
| `swissknife-ecommerce-sdk` | E-commerce (Shopify, WooCommerce, BigCommerce) |

### Infrastructure & Data

| Crate | Description |
|-------|-------------|
| `swissknife-database-sdk` | Databases (Supabase, PlanetScale, Turso, Neon, MongoDB, PostgreSQL, Redis) |
| `swissknife-cloud-sdk` | Cloud providers (AWS, GCP, Azure, Vercel, Cloudflare) |
| `swissknife-queue-sdk` | Message queues (SQS, RabbitMQ, Kafka) |
| `swissknife-file-sdk` | File storage (S3, GCS, SFTP, SSH) |
| `swissknife-observability-sdk` | Observability (Datadog, PagerDuty, incident.io) |

### Security & Auth

| Crate | Description |
|-------|-------------|
| `swissknife-auth-sdk` | Authentication (OAuth2, JWT, SAML, LDAP, SCIM, WebAuthn, TOTP) |

### Development

| Crate | Description |
|-------|-------------|
| `swissknife-devtools-sdk` | Dev tools (GitHub, GitLab, Cursor, Stagehand) |
| `swissknife-research-sdk` | Research (arXiv, Semantic Scholar) |

## MCP Server

The `swissknife-ai-sdk` includes a Model Context Protocol (MCP) server that exposes tools for AI agents. Built on the official [rmcp](https://github.com/modelcontextprotocol/rust-sdk) crate.

### Available MCP Tools

- **Search**: Tavily, Exa, Serper, Perplexity, DuckDuckGo, Wikipedia
- **LLM**: OpenAI (chat, embeddings, images), Anthropic, Mistral, DeepL translation, Stability AI
- **Communication**: Slack, Discord, Telegram, SendGrid, Resend, Twilio
- **Productivity**: Notion, Airtable, Google Drive/Calendar
- **Database**: Supabase, PlanetScale, Turso, Neon, Upstash, Pinecone, Qdrant, Weaviate, ChromaDB
- **Memory**: Mem0, Zep
- **Scraping**: Apify, Firecrawl, BrowserUse

### Usage

```rust
use swissknife_ai_sdk::mcp::{serve_stdio, tools::SearchTools};
use swissknife_search_sdk::tavily::TavilyClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tavily = TavilyClient::new("your-api-key");
    let tools = SearchTools::new().with_tavily(tavily);

    serve_stdio(tools).await?;
    Ok(())
}
```

## Installation

```toml
[dependencies]
# AI SDK with MCP support
swissknife-ai-sdk = { version = "0.1", features = ["mcp", "openai", "anthropic"] }

# Individual SDKs
swissknife-search-sdk = { version = "0.1", features = ["tavily", "exa"] }
swissknife-communication-sdk = { version = "0.1", features = ["slack", "discord"] }
swissknife-database-sdk = { version = "0.1", features = ["supabase", "pinecone"] }
```

## Example

```rust
use swissknife_ai_sdk::llm::{ChatProvider, ChatRequest, ChatMessage};
use swissknife_ai_sdk::llm::openai::OpenAIClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenAIClient::new("your-api-key");

    let request = ChatRequest::new(
        "gpt-4",
        vec![ChatMessage::user("Hello, world!")],
    );

    let response = client.chat(&request).await?;
    println!("{}", response.content().unwrap_or_default());

    Ok(())
}
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
