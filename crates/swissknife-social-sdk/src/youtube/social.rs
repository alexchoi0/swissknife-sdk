use crate::{Error, Result};
use crate::youtube::YouTubeClient;
use serde::{Deserialize, Serialize};

impl YouTubeClient {
    pub async fn search(&self, query: &str, options: SearchOptions) -> Result<SearchResponse> {
        let mut params = vec![
            ("part", "snippet".to_string()),
            ("q", query.to_string()),
            ("maxResults", options.max_results.unwrap_or(25).to_string()),
        ];

        if let Some(search_type) = options.search_type {
            params.push(("type", search_type));
        }
        if let Some(channel_id) = options.channel_id {
            params.push(("channelId", channel_id));
        }
        if let Some(order) = options.order {
            params.push(("order", order));
        }

        let request = self.client()
            .get(format!("{}/search", self.base_url()))
            .query(&params);

        let response = self.add_auth(request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let search_response: SearchResponse = response.json().await?;
        Ok(search_response)
    }

    pub async fn get_video(&self, video_id: &str) -> Result<Video> {
        let params = vec![
            ("part", "snippet,contentDetails,statistics"),
            ("id", video_id),
        ];

        let request = self.client()
            .get(format!("{}/videos", self.base_url()))
            .query(&params);

        let response = self.add_auth(request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let videos_response: VideosResponse = response.json().await?;
        videos_response.items.into_iter().next()
            .ok_or_else(|| Error::NotFound(video_id.to_string()))
    }

    pub async fn get_channel(&self, channel_id: &str) -> Result<Channel> {
        let params = vec![
            ("part", "snippet,contentDetails,statistics"),
            ("id", channel_id),
        ];

        let request = self.client()
            .get(format!("{}/channels", self.base_url()))
            .query(&params);

        let response = self.add_auth(request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let channels_response: ChannelsResponse = response.json().await?;
        channels_response.items.into_iter().next()
            .ok_or_else(|| Error::NotFound(channel_id.to_string()))
    }

    pub async fn get_playlist(&self, playlist_id: &str) -> Result<Playlist> {
        let params = vec![
            ("part", "snippet,contentDetails"),
            ("id", playlist_id),
        ];

        let request = self.client()
            .get(format!("{}/playlists", self.base_url()))
            .query(&params);

        let response = self.add_auth(request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let playlists_response: PlaylistsResponse = response.json().await?;
        playlists_response.items.into_iter().next()
            .ok_or_else(|| Error::NotFound(playlist_id.to_string()))
    }

    pub async fn get_playlist_items(&self, playlist_id: &str, max_results: Option<u32>) -> Result<PlaylistItemsResponse> {
        let params = vec![
            ("part", "snippet,contentDetails".to_string()),
            ("playlistId", playlist_id.to_string()),
            ("maxResults", max_results.unwrap_or(50).to_string()),
        ];

        let request = self.client()
            .get(format!("{}/playlistItems", self.base_url()))
            .query(&params);

        let response = self.add_auth(request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let items_response: PlaylistItemsResponse = response.json().await?;
        Ok(items_response)
    }

    pub async fn get_comments(&self, video_id: &str, max_results: Option<u32>) -> Result<CommentThreadsResponse> {
        let params = vec![
            ("part", "snippet".to_string()),
            ("videoId", video_id.to_string()),
            ("maxResults", max_results.unwrap_or(20).to_string()),
        ];

        let request = self.client()
            .get(format!("{}/commentThreads", self.base_url()))
            .query(&params);

        let response = self.add_auth(request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let comments_response: CommentThreadsResponse = response.json().await?;
        Ok(comments_response)
    }

    pub async fn get_my_channel(&self) -> Result<Channel> {
        if self.access_token.is_none() {
            return Err(Error::Auth("OAuth token required for this operation".to_string()));
        }

        let params = vec![
            ("part", "snippet,contentDetails,statistics"),
            ("mine", "true"),
        ];

        let request = self.client()
            .get(format!("{}/channels", self.base_url()))
            .query(&params);

        let response = self.add_auth(request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let channels_response: ChannelsResponse = response.json().await?;
        channels_response.items.into_iter().next()
            .ok_or_else(|| Error::NotFound("my channel".to_string()))
    }

    pub async fn get_my_subscriptions(&self, max_results: Option<u32>) -> Result<SubscriptionsResponse> {
        if self.access_token.is_none() {
            return Err(Error::Auth("OAuth token required for this operation".to_string()));
        }

        let params = vec![
            ("part", "snippet".to_string()),
            ("mine", "true".to_string()),
            ("maxResults", max_results.unwrap_or(25).to_string()),
        ];

        let request = self.client()
            .get(format!("{}/subscriptions", self.base_url()))
            .query(&params);

        let response = self.add_auth(request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let subs_response: SubscriptionsResponse = response.json().await?;
        Ok(subs_response)
    }
}

#[derive(Default)]
pub struct SearchOptions {
    pub search_type: Option<String>,
    pub channel_id: Option<String>,
    pub order: Option<String>,
    pub max_results: Option<u32>,
}

impl SearchOptions {
    pub fn videos() -> Self {
        Self {
            search_type: Some("video".to_string()),
            ..Default::default()
        }
    }

    pub fn channels() -> Self {
        Self {
            search_type: Some("channel".to_string()),
            ..Default::default()
        }
    }

    pub fn playlists() -> Self {
        Self {
            search_type: Some("playlist".to_string()),
            ..Default::default()
        }
    }

    pub fn max_results(mut self, max: u32) -> Self {
        self.max_results = Some(max);
        self
    }

    pub fn order_by(mut self, order: &str) -> Self {
        self.order = Some(order.to_string());
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    pub items: Vec<SearchResult>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    #[serde(rename = "pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult {
    pub id: SearchResultId,
    pub snippet: Snippet,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResultId {
    pub kind: String,
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
    #[serde(rename = "channelId")]
    pub channel_id: Option<String>,
    #[serde(rename = "playlistId")]
    pub playlist_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Snippet {
    pub title: String,
    pub description: Option<String>,
    #[serde(rename = "publishedAt")]
    pub published_at: Option<String>,
    #[serde(rename = "channelId")]
    pub channel_id: Option<String>,
    #[serde(rename = "channelTitle")]
    pub channel_title: Option<String>,
    pub thumbnails: Option<Thumbnails>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Thumbnails {
    pub default: Option<Thumbnail>,
    pub medium: Option<Thumbnail>,
    pub high: Option<Thumbnail>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Thumbnail {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageInfo {
    #[serde(rename = "totalResults")]
    pub total_results: u64,
    #[serde(rename = "resultsPerPage")]
    pub results_per_page: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VideosResponse {
    pub items: Vec<Video>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Video {
    pub id: String,
    pub snippet: Snippet,
    #[serde(rename = "contentDetails")]
    pub content_details: Option<VideoContentDetails>,
    pub statistics: Option<VideoStatistics>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VideoContentDetails {
    pub duration: Option<String>,
    pub definition: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VideoStatistics {
    #[serde(rename = "viewCount")]
    pub view_count: Option<String>,
    #[serde(rename = "likeCount")]
    pub like_count: Option<String>,
    #[serde(rename = "commentCount")]
    pub comment_count: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChannelsResponse {
    pub items: Vec<Channel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Channel {
    pub id: String,
    pub snippet: ChannelSnippet,
    #[serde(rename = "contentDetails")]
    pub content_details: Option<ChannelContentDetails>,
    pub statistics: Option<ChannelStatistics>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChannelSnippet {
    pub title: String,
    pub description: Option<String>,
    #[serde(rename = "customUrl")]
    pub custom_url: Option<String>,
    #[serde(rename = "publishedAt")]
    pub published_at: Option<String>,
    pub thumbnails: Option<Thumbnails>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChannelContentDetails {
    #[serde(rename = "relatedPlaylists")]
    pub related_playlists: Option<RelatedPlaylists>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RelatedPlaylists {
    pub uploads: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChannelStatistics {
    #[serde(rename = "viewCount")]
    pub view_count: Option<String>,
    #[serde(rename = "subscriberCount")]
    pub subscriber_count: Option<String>,
    #[serde(rename = "videoCount")]
    pub video_count: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistsResponse {
    pub items: Vec<Playlist>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub snippet: Snippet,
    #[serde(rename = "contentDetails")]
    pub content_details: Option<PlaylistContentDetails>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistContentDetails {
    #[serde(rename = "itemCount")]
    pub item_count: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistItemsResponse {
    pub items: Vec<PlaylistItem>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistItem {
    pub id: String,
    pub snippet: PlaylistItemSnippet,
    #[serde(rename = "contentDetails")]
    pub content_details: Option<PlaylistItemContentDetails>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistItemSnippet {
    pub title: String,
    pub description: Option<String>,
    pub position: Option<u32>,
    #[serde(rename = "resourceId")]
    pub resource_id: Option<ResourceId>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceId {
    pub kind: String,
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistItemContentDetails {
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
    #[serde(rename = "videoPublishedAt")]
    pub video_published_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommentThreadsResponse {
    pub items: Vec<CommentThread>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommentThread {
    pub id: String,
    pub snippet: CommentThreadSnippet,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommentThreadSnippet {
    #[serde(rename = "topLevelComment")]
    pub top_level_comment: Comment,
    #[serde(rename = "totalReplyCount")]
    pub total_reply_count: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Comment {
    pub id: String,
    pub snippet: CommentSnippet,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommentSnippet {
    #[serde(rename = "authorDisplayName")]
    pub author_display_name: String,
    #[serde(rename = "textDisplay")]
    pub text_display: String,
    #[serde(rename = "likeCount")]
    pub like_count: Option<u32>,
    #[serde(rename = "publishedAt")]
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionsResponse {
    pub items: Vec<Subscription>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub snippet: SubscriptionSnippet,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionSnippet {
    pub title: String,
    pub description: Option<String>,
    #[serde(rename = "channelId")]
    pub channel_id: String,
    pub thumbnails: Option<Thumbnails>,
}
