mod error;

#[cfg(feature = "instagram")]
pub mod instagram;

#[cfg(feature = "facebook")]
pub mod facebook;

#[cfg(feature = "tiktok")]
pub mod tiktok;

#[cfg(feature = "twitter")]
pub mod twitter;

#[cfg(feature = "linkedin")]
pub mod linkedin;

#[cfg(feature = "reddit")]
pub mod reddit;

#[cfg(feature = "spotify")]
pub mod spotify;

#[cfg(feature = "youtube")]
pub mod youtube;

pub use error::{Error, Result};

#[cfg(feature = "social")]
pub mod social {
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum MediaType {
        Image,
        Video,
        Carousel,
        Reel,
        Story,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MediaItem {
        pub url: String,
        pub media_type: MediaType,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub thumbnail_url: Option<String>,
    }

    impl MediaItem {
        pub fn image(url: impl Into<String>) -> Self {
            Self {
                url: url.into(),
                media_type: MediaType::Image,
                thumbnail_url: None,
            }
        }

        pub fn video(url: impl Into<String>) -> Self {
            Self {
                url: url.into(),
                media_type: MediaType::Video,
                thumbnail_url: None,
            }
        }

        pub fn reel(url: impl Into<String>) -> Self {
            Self {
                url: url.into(),
                media_type: MediaType::Reel,
                thumbnail_url: None,
            }
        }

        pub fn story(url: impl Into<String>) -> Self {
            Self {
                url: url.into(),
                media_type: MediaType::Story,
                thumbnail_url: None,
            }
        }

        pub fn thumbnail(mut self, url: impl Into<String>) -> Self {
            self.thumbnail_url = Some(url.into());
            self
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SocialPost {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub text: Option<String>,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        pub media: Vec<MediaItem>,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        pub hashtags: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub link: Option<String>,
    }

    impl SocialPost {
        pub fn new() -> Self {
            Self {
                text: None,
                media: Vec::new(),
                hashtags: Vec::new(),
                link: None,
            }
        }

        pub fn text(mut self, text: impl Into<String>) -> Self {
            self.text = Some(text.into());
            self
        }

        pub fn media(mut self, item: MediaItem) -> Self {
            self.media.push(item);
            self
        }

        pub fn hashtag(mut self, tag: impl Into<String>) -> Self {
            self.hashtags.push(tag.into());
            self
        }

        pub fn link(mut self, url: impl Into<String>) -> Self {
            self.link = Some(url.into());
            self
        }

        pub fn build_caption(&self) -> String {
            let mut caption = self.text.clone().unwrap_or_default();
            if !self.hashtags.is_empty() {
                if !caption.is_empty() {
                    caption.push_str("\n\n");
                }
                caption.push_str(
                    &self
                        .hashtags
                        .iter()
                        .map(|t| format!("#{}", t.trim_start_matches('#')))
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
            caption
        }
    }

    impl Default for SocialPost {
        fn default() -> Self {
            Self::new()
        }
    }

    #[derive(Debug, Clone)]
    pub struct PostResponse {
        pub post_id: String,
        pub url: Option<String>,
        pub status: String,
    }

    #[async_trait]
    pub trait SocialPoster: Send + Sync {
        async fn create_post(&self, post: &SocialPost) -> crate::Result<PostResponse>;
    }

    #[async_trait]
    pub trait SocialMediaUploader: Send + Sync {
        async fn upload_media(&self, media: &MediaItem) -> crate::Result<String>;
    }
}
