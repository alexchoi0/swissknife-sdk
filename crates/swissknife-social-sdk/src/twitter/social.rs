use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::social::{PostResponse, SocialPost, SocialPoster};
use crate::{Error, Result};

use super::TwitterClient;

#[derive(Serialize)]
struct TweetRequest {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    media: Option<TweetMedia>,
}

#[derive(Serialize)]
struct TweetMedia {
    media_ids: Vec<String>,
}

#[derive(Deserialize)]
struct TweetResponse {
    data: Option<TweetData>,
    errors: Option<Vec<TweetError>>,
}

#[derive(Deserialize)]
struct TweetData {
    id: String,
    text: String,
}

#[derive(Deserialize)]
struct TweetError {
    message: String,
    #[serde(default)]
    code: Option<i32>,
}

#[async_trait]
impl SocialPoster for TwitterClient {
    async fn create_post(&self, post: &SocialPost) -> Result<PostResponse> {
        let text = post.build_caption();

        if text.is_empty() && post.media.is_empty() {
            return Err(Error::Config("Tweet requires text or media".into()));
        }

        let request = TweetRequest {
            text,
            media: None,
        };

        let response = self
            .http
            .post(&self.tweets_url())
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let result: TweetResponse = response.json().await?;

        if let Some(errors) = result.errors {
            if let Some(err) = errors.first() {
                return Err(Error::Api {
                    code: err.code.unwrap_or(status.as_u16() as i32),
                    message: err.message.clone(),
                });
            }
        }

        let data = result.data.ok_or_else(|| Error::Api {
            code: 500,
            message: "No data in response".into(),
        })?;

        Ok(PostResponse {
            post_id: data.id.clone(),
            url: Some(format!("https://twitter.com/i/status/{}", data.id)),
            status: "published".into(),
        })
    }
}
