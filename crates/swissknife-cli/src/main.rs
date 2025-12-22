use clap::{Parser, Subcommand};
use swissknife_communication_sdk::chat::{ChatMessage, ChatSender};
use swissknife_communication_sdk::discord::{DiscordBotClient, DiscordWebhookClient};
use swissknife_communication_sdk::email::{Email, EmailAddress, EmailSender};
use swissknife_communication_sdk::fcm::FcmClient;
use swissknife_communication_sdk::push::{PushNotification, PushSender};
use swissknife_communication_sdk::sendgrid::SendGridClient;
use swissknife_communication_sdk::slack::{SlackClient, SlackWebhookClient};
use swissknife_communication_sdk::sms::SmsSender;
use swissknife_communication_sdk::telegram::TelegramClient;
use swissknife_communication_sdk::twilio::TwilioClient;
use swissknife_communication_sdk::voice::VoiceCaller;
use swissknife_communication_sdk::whatsapp::WhatsAppSender;
use swissknife_social_sdk::facebook::FacebookClient;
use swissknife_social_sdk::instagram::InstagramClient;
use swissknife_social_sdk::linkedin::LinkedInClient;
use swissknife_social_sdk::social::{MediaItem, SocialPost, SocialPoster};
use swissknife_social_sdk::tiktok::TikTokClient;
use swissknife_social_sdk::twitter::TwitterClient;
use swissknife_auth_sdk::password::{PasswordHasher, verify as verify_password, check_strength};
use swissknife_auth_sdk::api_keys::{ApiKeyGenerator, ApiKeyConfig};
use swissknife_auth_sdk::totp::{TotpConfig, TotpSecret};
use swissknife_auth_sdk::jwt::{JwtEncoder, JwtDecoder, Claims};

#[derive(Parser)]
#[command(name = "swissknife")]
#[command(about = "CLI for testing swissknife SDKs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Sms {
        #[arg(long, env = "TWILIO_ACCOUNT_SID")]
        account_sid: String,
        #[arg(long, env = "TWILIO_AUTH_TOKEN")]
        auth_token: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(long)]
        body: String,
    },
    Call {
        #[arg(long, env = "TWILIO_ACCOUNT_SID")]
        account_sid: String,
        #[arg(long, env = "TWILIO_AUTH_TOKEN")]
        auth_token: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(long)]
        twiml_url: String,
    },
    Whatsapp {
        #[arg(long, env = "TWILIO_ACCOUNT_SID")]
        account_sid: String,
        #[arg(long, env = "TWILIO_AUTH_TOKEN")]
        auth_token: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(long)]
        body: String,
    },
    Email {
        #[arg(long, env = "SENDGRID_API_KEY")]
        api_key: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        from_name: Option<String>,
        #[arg(long)]
        to: String,
        #[arg(long)]
        to_name: Option<String>,
        #[arg(long)]
        subject: String,
        #[arg(long)]
        body: String,
        #[arg(long)]
        html: bool,
    },
    Push {
        #[command(subcommand)]
        provider: PushProvider,
    },
    Chat {
        #[command(subcommand)]
        provider: ChatProvider,
    },
    Social {
        #[command(subcommand)]
        provider: SocialProvider,
    },
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
}

#[derive(Subcommand)]
enum PushProvider {
    Fcm {
        #[arg(long, env = "FCM_PROJECT_ID")]
        project_id: String,
        #[arg(long, env = "FCM_ACCESS_TOKEN")]
        access_token: String,
        #[arg(long)]
        token: Option<String>,
        #[arg(long)]
        topic: Option<String>,
        #[arg(long)]
        title: String,
        #[arg(long)]
        body: String,
    },
}

#[derive(Subcommand)]
enum ChatProvider {
    Slack {
        #[arg(long, env = "SLACK_BOT_TOKEN")]
        bot_token: Option<String>,
        #[arg(long, env = "SLACK_WEBHOOK_URL")]
        webhook_url: Option<String>,
        #[arg(long)]
        channel: String,
        #[arg(long)]
        text: String,
        #[arg(long)]
        username: Option<String>,
    },
    Discord {
        #[arg(long, env = "DISCORD_BOT_TOKEN")]
        bot_token: Option<String>,
        #[arg(long, env = "DISCORD_WEBHOOK_URL")]
        webhook_url: Option<String>,
        #[arg(long)]
        channel: String,
        #[arg(long)]
        text: String,
        #[arg(long)]
        username: Option<String>,
    },
    Telegram {
        #[arg(long, env = "TELEGRAM_BOT_TOKEN")]
        bot_token: String,
        #[arg(long)]
        chat_id: String,
        #[arg(long)]
        text: String,
    },
}

#[derive(Subcommand)]
enum SocialProvider {
    Instagram {
        #[arg(long, env = "INSTAGRAM_ACCESS_TOKEN")]
        access_token: String,
        #[arg(long, env = "INSTAGRAM_ACCOUNT_ID")]
        account_id: String,
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        image_url: Option<String>,
        #[arg(long)]
        video_url: Option<String>,
        #[arg(long, num_args = 0..)]
        hashtags: Vec<String>,
    },
    Facebook {
        #[arg(long, env = "FACEBOOK_ACCESS_TOKEN")]
        access_token: String,
        #[arg(long, env = "FACEBOOK_PAGE_ID")]
        page_id: String,
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        image_url: Option<String>,
        #[arg(long)]
        link: Option<String>,
        #[arg(long, num_args = 0..)]
        hashtags: Vec<String>,
    },
    Tiktok {
        #[arg(long, env = "TIKTOK_ACCESS_TOKEN")]
        access_token: String,
        #[arg(long)]
        video_url: String,
        #[arg(long)]
        text: Option<String>,
        #[arg(long, num_args = 0..)]
        hashtags: Vec<String>,
    },
    Twitter {
        #[arg(long, env = "TWITTER_BEARER_TOKEN")]
        bearer_token: String,
        #[arg(long)]
        text: String,
        #[arg(long, num_args = 0..)]
        hashtags: Vec<String>,
    },
    Linkedin {
        #[arg(long, env = "LINKEDIN_ACCESS_TOKEN")]
        access_token: String,
        #[arg(long, env = "LINKEDIN_PERSON_URN")]
        person_urn: String,
        #[arg(long)]
        text: String,
        #[arg(long, num_args = 0..)]
        hashtags: Vec<String>,
    },
}

#[derive(Subcommand)]
enum AuthAction {
    HashPassword {
        #[arg(long)]
        password: String,
        #[arg(long, default_value = "argon2id")]
        algorithm: String,
    },
    VerifyPassword {
        #[arg(long)]
        password: String,
        #[arg(long)]
        hash: String,
    },
    CheckStrength {
        #[arg(long)]
        password: String,
    },
    GenerateApiKey {
        #[arg(long, default_value = "sk")]
        prefix: String,
    },
    VerifyApiKey {
        #[arg(long)]
        key: String,
        #[arg(long)]
        hash: String,
    },
    GenerateTotp {
        #[arg(long)]
        issuer: String,
        #[arg(long)]
        account: String,
    },
    VerifyTotp {
        #[arg(long)]
        secret: String,
        #[arg(long)]
        code: String,
        #[arg(long)]
        issuer: String,
        #[arg(long)]
        account: String,
    },
    GenerateJwt {
        #[arg(long)]
        subject: String,
        #[arg(long)]
        secret: String,
        #[arg(long, default_value = "3600")]
        expires_in: i64,
    },
    VerifyJwt {
        #[arg(long)]
        token: String,
        #[arg(long)]
        secret: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sms { account_sid, auth_token, from, to, body } => {
            let client = TwilioClient::new(account_sid, auth_token)?;
            let response = client.send_sms(&from, &to, &body).await?;
            println!("SMS sent!");
            println!("  ID: {}", response.message_id);
            println!("  Status: {}", response.status);
        }
        Commands::Call { account_sid, auth_token, from, to, twiml_url } => {
            let client = TwilioClient::new(account_sid, auth_token)?;
            let response = client.make_call(&from, &to, &twiml_url).await?;
            println!("Call initiated!");
            println!("  ID: {}", response.call_id);
            println!("  Status: {}", response.status);
        }
        Commands::Whatsapp { account_sid, auth_token, from, to, body } => {
            let client = TwilioClient::new(account_sid, auth_token)?;
            let response = client.send_whatsapp(&from, &to, &body).await?;
            println!("WhatsApp sent!");
            println!("  ID: {}", response.message_id);
            println!("  Status: {}", response.status);
        }
        Commands::Email { api_key, from, from_name, to, to_name, subject, body, html } => {
            let client = SendGridClient::new(api_key)?;
            let from_addr = match from_name {
                Some(name) => EmailAddress::with_name(from, name),
                None => EmailAddress::new(from),
            };
            let to_addr = match to_name {
                Some(name) => EmailAddress::with_name(to, name),
                None => EmailAddress::new(to),
            };
            let mut email = Email::new(from_addr, to_addr, subject);
            email = if html { email.html(body) } else { email.text(body) };
            let response = client.send_email(&email).await?;
            println!("Email sent!");
            if let Some(id) = response.message_id { println!("  ID: {}", id); }
            println!("  Status: {}", response.status);
        }
        Commands::Push { provider } => match provider {
            PushProvider::Fcm { project_id, access_token, token, topic, title, body } => {
                let client = FcmClient::new(project_id, access_token)?;
                let notification = PushNotification::new(title, body);
                let response = if let Some(t) = token {
                    client.send_to_token(&t, &notification).await?
                } else if let Some(t) = topic {
                    client.send_to_topic(&t, &notification).await?
                } else {
                    return Err("--token or --topic required".into());
                };
                println!("Push sent!");
                if let Some(id) = response.message_id { println!("  ID: {}", id); }
                println!("  Success: {}, Failures: {}", response.success_count, response.failure_count);
            }
        },
        Commands::Chat { provider } => match provider {
            ChatProvider::Slack { bot_token, webhook_url, channel, text, username } => {
                let mut message = ChatMessage::new(text);
                if let Some(n) = username { message = message.username(n); }
                let response = if let Some(t) = bot_token {
                    SlackClient::new(t)?.send_message(&channel, &message).await?
                } else if let Some(u) = webhook_url {
                    SlackWebhookClient::new(u)?.send_message(&channel, &message).await?
                } else {
                    return Err("--bot-token or --webhook-url required".into());
                };
                println!("Slack sent!");
                if let Some(id) = response.message_id { println!("  ID: {}", id); }
            }
            ChatProvider::Discord { bot_token, webhook_url, channel, text, username } => {
                let mut message = ChatMessage::new(text);
                if let Some(n) = username { message = message.username(n); }
                let response = if let Some(t) = bot_token {
                    DiscordBotClient::new(t)?.send_message(&channel, &message).await?
                } else if let Some(u) = webhook_url {
                    DiscordWebhookClient::new(u)?.send_message(&channel, &message).await?
                } else {
                    return Err("--bot-token or --webhook-url required".into());
                };
                println!("Discord sent!");
                if let Some(id) = response.message_id { println!("  ID: {}", id); }
            }
            ChatProvider::Telegram { bot_token, chat_id, text } => {
                let client = TelegramClient::new(bot_token)?;
                let response = client.send_message(&chat_id, &ChatMessage::new(text)).await?;
                println!("Telegram sent!");
                if let Some(id) = response.message_id { println!("  ID: {}", id); }
            }
        },
        Commands::Social { provider } => match provider {
            SocialProvider::Instagram { access_token, account_id, text, image_url, video_url, hashtags } => {
                let client = InstagramClient::new(access_token, account_id)?;
                let mut post = SocialPost::new();
                if let Some(t) = text { post = post.text(t); }
                if let Some(url) = image_url { post = post.media(MediaItem::image(url)); }
                if let Some(url) = video_url { post = post.media(MediaItem::video(url)); }
                for tag in hashtags { post = post.hashtag(tag); }
                let response = client.create_post(&post).await?;
                println!("Instagram posted!");
                println!("  ID: {}", response.post_id);
                if let Some(url) = response.url { println!("  URL: {}", url); }
            }
            SocialProvider::Facebook { access_token, page_id, text, image_url, link, hashtags } => {
                let client = FacebookClient::new(access_token, page_id)?;
                let mut post = SocialPost::new();
                if let Some(t) = text { post = post.text(t); }
                if let Some(url) = image_url { post = post.media(MediaItem::image(url)); }
                if let Some(l) = link { post = post.link(l); }
                for tag in hashtags { post = post.hashtag(tag); }
                let response = client.create_post(&post).await?;
                println!("Facebook posted!");
                println!("  ID: {}", response.post_id);
                if let Some(url) = response.url { println!("  URL: {}", url); }
            }
            SocialProvider::Tiktok { access_token, video_url, text, hashtags } => {
                let client = TikTokClient::new(access_token)?;
                let mut post = SocialPost::new().media(MediaItem::video(video_url));
                if let Some(t) = text { post = post.text(t); }
                for tag in hashtags { post = post.hashtag(tag); }
                let response = client.create_post(&post).await?;
                println!("TikTok posted!");
                println!("  ID: {}", response.post_id);
                println!("  Status: {}", response.status);
            }
            SocialProvider::Twitter { bearer_token, text, hashtags } => {
                let client = TwitterClient::new(bearer_token)?;
                let mut post = SocialPost::new().text(text);
                for tag in hashtags { post = post.hashtag(tag); }
                let response = client.create_post(&post).await?;
                println!("Tweet posted!");
                println!("  ID: {}", response.post_id);
                if let Some(url) = response.url { println!("  URL: {}", url); }
            }
            SocialProvider::Linkedin { access_token, person_urn, text, hashtags } => {
                let client = LinkedInClient::new(access_token, person_urn)?;
                let mut post = SocialPost::new().text(text);
                for tag in hashtags { post = post.hashtag(tag); }
                let response = client.create_post(&post).await?;
                println!("LinkedIn posted!");
                println!("  ID: {}", response.post_id);
                if let Some(url) = response.url { println!("  URL: {}", url); }
            }
        },
        Commands::Auth { action } => match action {
            AuthAction::HashPassword { password, algorithm } => {
                let hasher = match algorithm.as_str() {
                    "argon2id" => PasswordHasher::argon2id(),
                    "bcrypt" => PasswordHasher::bcrypt(),
                    "scrypt" => PasswordHasher::scrypt(),
                    _ => return Err(format!("Unknown algorithm: {}", algorithm).into()),
                };
                let hash = hasher.hash(&password)?;
                println!("Password hashed!");
                println!("  Hash: {}", hash);
            }
            AuthAction::VerifyPassword { password, hash } => {
                let valid = verify_password(&password, &hash)?;
                println!("Password verification: {}", if valid { "VALID" } else { "INVALID" });
            }
            AuthAction::CheckStrength { password } => {
                let strength = check_strength(&password);
                println!("Password strength: {}/5", strength.score);
                if !strength.feedback.is_empty() {
                    println!("Suggestions:");
                    for feedback in strength.feedback {
                        println!("  - {}", feedback);
                    }
                }
            }
            AuthAction::GenerateApiKey { prefix } => {
                let generator = ApiKeyGenerator::new(ApiKeyConfig::new(prefix));
                let api_key = generator.generate();
                println!("API Key generated!");
                println!("  Key: {}", api_key.full_key);
                println!("  Key ID: {}", api_key.key_id);
                println!("  Prefix: {}", api_key.prefix);
                println!("  Hash: {}", api_key.hash);
            }
            AuthAction::VerifyApiKey { key, hash } => {
                let generator = ApiKeyGenerator::new(ApiKeyConfig::default());
                let valid = generator.verify(&key, &hash);
                println!("API Key verification: {}", if valid { "VALID" } else { "INVALID" });
            }
            AuthAction::GenerateTotp { issuer, account } => {
                let config = TotpConfig::new(&issuer, &account);
                let secret = TotpSecret::generate(&config)?;
                println!("TOTP generated!");
                println!("  Secret: {}", secret.secret_base32());
                println!("  URI: {}", secret.otpauth_uri());
                println!("  Current code: {}", secret.generate_code()?);
            }
            AuthAction::VerifyTotp { secret, code, issuer, account } => {
                let config = TotpConfig::new(&issuer, &account);
                let totp = TotpSecret::from_base32(&secret, &config)?;
                let valid = totp.verify(&code)?;
                println!("TOTP verification: {}", if valid { "VALID" } else { "INVALID" });
            }
            AuthAction::GenerateJwt { subject, secret, expires_in } => {
                let encoder = JwtEncoder::hs256(secret.as_bytes());
                let claims = Claims::new(&subject, expires_in);
                let token = encoder.encode(&claims)?;
                println!("JWT generated!");
                println!("  Token: {}", token);
            }
            AuthAction::VerifyJwt { token, secret } => {
                let decoder = JwtDecoder::hs256(secret.as_bytes());
                match decoder.decode_claims(&token) {
                    Ok(data) => {
                        println!("JWT verification: VALID");
                        println!("  Subject: {}", data.claims.sub);
                        println!("  Expires: {}", data.claims.exp);
                    }
                    Err(e) => {
                        println!("JWT verification: INVALID");
                        println!("  Error: {}", e);
                    }
                }
            }
        },
    }

    Ok(())
}
