use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::social::{PostResponse, SocialPost, SocialPoster};
use crate::{Error, Result};

use super::LinkedInClient;

#[derive(Serialize)]
struct LinkedInPostRequest {
    author: String,
    commentary: String,
    visibility: String,
    distribution: Distribution,
    #[serde(rename = "lifecycleState")]
    lifecycle_state: String,
}

#[derive(Serialize)]
struct Distribution {
    #[serde(rename = "feedDistribution")]
    feed_distribution: String,
}

#[derive(Deserialize)]
struct LinkedInResponse {
    id: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    status: Option<i32>,
}

#[async_trait]
impl SocialPoster for LinkedInClient {
    async fn create_post(&self, post: &SocialPost) -> Result<PostResponse> {
        let commentary = post.build_caption();

        if commentary.is_empty() {
            return Err(Error::Config("LinkedIn post requires text".into()));
        }

        let request = LinkedInPostRequest {
            author: self.person_urn.clone(),
            commentary,
            visibility: "PUBLIC".into(),
            distribution: Distribution {
                feed_distribution: "MAIN_FEED".into(),
            },
            lifecycle_state: "PUBLISHED".into(),
        };

        let response = self
            .http
            .post(&self.posts_url())
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .header("X-Restli-Protocol-Version", "2.0.0")
            .header("LinkedIn-Version", "202401")
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let result: LinkedInResponse = response.json().await.unwrap_or(LinkedInResponse {
                id: None,
                message: Some("Unknown error".into()),
                status: Some(status.as_u16() as i32),
            });
            return Err(Error::Api {
                code: result.status.unwrap_or(status.as_u16() as i32),
                message: result.message.unwrap_or_else(|| "Unknown error".into()),
            });
        }

        let post_id = response
            .headers()
            .get("x-restli-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".into());

        Ok(PostResponse {
            post_id: post_id.clone(),
            url: Some(format!("https://www.linkedin.com/feed/update/{}", post_id)),
            status: "published".into(),
        })
    }
}
