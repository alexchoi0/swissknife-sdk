use async_trait::async_trait;
use serde::Deserialize;

use crate::social::{MediaType, PostResponse, SocialPost, SocialPoster};
use crate::{Error, Result};

use super::InstagramClient;

#[derive(Deserialize)]
struct ContainerResponse {
    id: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: GraphError,
}

#[derive(Deserialize)]
struct GraphError {
    message: String,
    code: i32,
}

impl InstagramClient {
    async fn create_container(
        &self,
        image_url: Option<&str>,
        video_url: Option<&str>,
        caption: Option<&str>,
        media_type: Option<&str>,
    ) -> Result<String> {
        let mut params = vec![("access_token", self.access_token.as_str())];

        if let Some(url) = image_url {
            params.push(("image_url", url));
        }
        if let Some(url) = video_url {
            params.push(("video_url", url));
        }
        if let Some(t) = media_type {
            params.push(("media_type", t));
        }
        if let Some(c) = caption {
            params.push(("caption", c));
        }

        let response = self.http.post(&self.media_url()).form(&params).send().await?;
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

        let result: ContainerResponse = serde_json::from_str(&body)?;
        Ok(result.id)
    }

    async fn publish(&self, creation_id: &str) -> Result<String> {
        let params = [
            ("access_token", self.access_token.as_str()),
            ("creation_id", creation_id),
        ];

        let response = self.http.post(&self.publish_url()).form(&params).send().await?;
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

        let result: ContainerResponse = serde_json::from_str(&body)?;
        Ok(result.id)
    }
}

#[async_trait]
impl SocialPoster for InstagramClient {
    async fn create_post(&self, post: &SocialPost) -> Result<PostResponse> {
        if post.media.is_empty() {
            return Err(Error::Config("Instagram requires at least one media item".into()));
        }

        let media = &post.media[0];
        let caption = post.build_caption();
        let caption_ref = if caption.is_empty() { None } else { Some(caption.as_str()) };

        let (image_url, video_url, media_type) = match media.media_type {
            MediaType::Image => (Some(media.url.as_str()), None, None),
            MediaType::Video => (None, Some(media.url.as_str()), Some("VIDEO")),
            MediaType::Reel => (None, Some(media.url.as_str()), Some("REELS")),
            MediaType::Story => (None, Some(media.url.as_str()), Some("STORIES")),
            _ => return Err(Error::Config("Unsupported media type".into())),
        };

        let container_id = self
            .create_container(image_url, video_url, caption_ref, media_type)
            .await?;

        let post_id = self.publish(&container_id).await?;

        Ok(PostResponse {
            post_id: post_id.clone(),
            url: Some(format!("https://www.instagram.com/p/{}/", post_id)),
            status: "published".into(),
        })
    }
}
