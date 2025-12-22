use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::social::{PostResponse, SocialPost, SocialPoster};
use crate::{Error, Result};

use super::TikTokClient;

#[derive(Serialize)]
struct TikTokPostRequest {
    post_info: PostInfo,
    source_info: SourceInfo,
}

#[derive(Serialize)]
struct PostInfo {
    title: String,
    privacy_level: String,
    disable_duet: bool,
    disable_comment: bool,
    disable_stitch: bool,
}

#[derive(Serialize)]
struct SourceInfo {
    source: String,
    video_url: Option<String>,
}

#[derive(Deserialize)]
struct TikTokResponse {
    data: Option<TikTokData>,
    error: Option<TikTokError>,
}

#[derive(Deserialize)]
struct TikTokData {
    publish_id: String,
}

#[derive(Deserialize)]
struct TikTokError {
    code: String,
    message: String,
}

#[async_trait]
impl SocialPoster for TikTokClient {
    async fn create_post(&self, post: &SocialPost) -> Result<PostResponse> {
        if post.media.is_empty() {
            return Err(Error::Config("TikTok requires a video".into()));
        }

        let media = &post.media[0];
        let caption = post.build_caption();

        let request = TikTokPostRequest {
            post_info: PostInfo {
                title: caption,
                privacy_level: "PUBLIC_TO_EVERYONE".into(),
                disable_duet: false,
                disable_comment: false,
                disable_stitch: false,
            },
            source_info: SourceInfo {
                source: "PULL_FROM_URL".into(),
                video_url: Some(media.url.clone()),
            },
        };

        let response = self
            .http
            .post(&self.post_publish_url())
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let result: TikTokResponse = response.json().await?;

        if let Some(err) = result.error {
            return Err(Error::Api {
                code: status.as_u16() as i32,
                message: format!("{}: {}", err.code, err.message),
            });
        }

        let data = result.data.ok_or_else(|| Error::Api {
            code: 500,
            message: "No data in response".into(),
        })?;

        Ok(PostResponse {
            post_id: data.publish_id,
            url: None,
            status: "processing".into(),
        })
    }
}
