use async_trait::async_trait;
use serde::Deserialize;

use crate::social::{MediaType, PostResponse, SocialPost, SocialPoster};
use crate::{Error, Result};

use super::FacebookClient;

#[derive(Deserialize)]
struct FbPostResponse {
    id: String,
    #[serde(default)]
    post_id: Option<String>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: FbError,
}

#[derive(Deserialize)]
struct FbError {
    message: String,
    code: i32,
}

#[async_trait]
impl SocialPoster for FacebookClient {
    async fn create_post(&self, post: &SocialPost) -> Result<PostResponse> {
        let message = post.build_caption();

        let (url, params) = if post.media.is_empty() {
            let mut p = vec![
                ("access_token", self.access_token.as_str()),
                ("message", message.as_str()),
            ];
            if let Some(link) = &post.link {
                p.push(("link", link.as_str()));
            }
            (self.feed_url(), p)
        } else {
            let media = &post.media[0];
            match media.media_type {
                MediaType::Image => {
                    let p = vec![
                        ("access_token", self.access_token.as_str()),
                        ("url", media.url.as_str()),
                        ("caption", message.as_str()),
                    ];
                    (self.photos_url(), p)
                }
                MediaType::Video | MediaType::Reel => {
                    let p = vec![
                        ("access_token", self.access_token.as_str()),
                        ("file_url", media.url.as_str()),
                        ("description", message.as_str()),
                    ];
                    (self.videos_url(), p)
                }
                _ => return Err(Error::Config("Unsupported media type".into())),
            }
        };

        let response = self.http.post(&url).form(&params).send().await?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&body) {
                return Err(Error::Api {
                    code: err.error.code,
                    message: err.error.message,
                });
            }
            return Err(Error::Api {
                code: status.as_u16() as i32,
                message: body,
            });
        }

        let result: FbPostResponse = serde_json::from_str(&body)?;
        let post_id = result.post_id.unwrap_or(result.id.clone());

        Ok(PostResponse {
            post_id: post_id.clone(),
            url: Some(format!("https://www.facebook.com/{}", post_id)),
            status: "published".into(),
        })
    }
}
