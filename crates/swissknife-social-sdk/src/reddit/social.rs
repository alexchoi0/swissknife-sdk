use crate::{Error, Result};
use crate::reddit::RedditClient;
use serde::{Deserialize, Serialize};

impl RedditClient {
    pub async fn me(&self) -> Result<RedditUser> {
        let response = self.client()
            .get(format!("{}/api/v1/me", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("User-Agent", self.user_agent())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let user: RedditUser = response.json().await?;
        Ok(user)
    }

    pub async fn submit_post(&self, subreddit: &str, post: SubmitPost) -> Result<SubmitResponse> {
        let mut form = vec![
            ("sr", subreddit.to_string()),
            ("title", post.title),
            ("kind", post.kind),
            ("api_type", "json".to_string()),
        ];

        if let Some(text) = post.text {
            form.push(("text", text));
        }
        if let Some(url) = post.url {
            form.push(("url", url));
        }
        if let Some(flair_id) = post.flair_id {
            form.push(("flair_id", flair_id));
        }
        if post.nsfw {
            form.push(("nsfw", "true".to_string()));
        }
        if post.spoiler {
            form.push(("spoiler", "true".to_string()));
        }

        let response = self.client()
            .post(format!("{}/api/submit", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("User-Agent", self.user_agent())
            .form(&form)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let submit_response: RedditSubmitWrapper = response.json().await?;
        Ok(submit_response.json.data)
    }

    pub async fn comment(&self, parent_fullname: &str, text: &str) -> Result<CommentResponse> {
        let form = vec![
            ("thing_id", parent_fullname),
            ("text", text),
            ("api_type", "json"),
        ];

        let response = self.client()
            .post(format!("{}/api/comment", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("User-Agent", self.user_agent())
            .form(&form)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let comment_response: RedditCommentWrapper = response.json().await?;
        Ok(comment_response.json.data)
    }

    pub async fn vote(&self, fullname: &str, direction: VoteDirection) -> Result<()> {
        let dir = match direction {
            VoteDirection::Up => "1",
            VoteDirection::Down => "-1",
            VoteDirection::None => "0",
        };

        let form = vec![
            ("id", fullname),
            ("dir", dir),
        ];

        let response = self.client()
            .post(format!("{}/api/vote", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("User-Agent", self.user_agent())
            .form(&form)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        Ok(())
    }

    pub async fn get_subreddit(&self, subreddit: &str, sort: &str, limit: u32) -> Result<SubredditListing> {
        let response = self.client()
            .get(format!("{}/r/{}/{}", self.base_url(), subreddit, sort))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("User-Agent", self.user_agent())
            .query(&[("limit", limit.to_string())])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let listing: SubredditListing = response.json().await?;
        Ok(listing)
    }

    pub async fn search(&self, query: &str, options: SearchOptions) -> Result<SubredditListing> {
        let mut params = vec![
            ("q", query.to_string()),
            ("limit", options.limit.unwrap_or(25).to_string()),
        ];

        if let Some(subreddit) = options.subreddit {
            params.push(("restrict_sr", "true".to_string()));
        }
        if let Some(sort) = options.sort {
            params.push(("sort", sort));
        }
        if let Some(time) = options.time {
            params.push(("t", time));
        }

        let url = if let Some(subreddit) = options.subreddit {
            format!("{}/r/{}/search", self.base_url(), subreddit)
        } else {
            format!("{}/search", self.base_url())
        };

        let response = self.client()
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("User-Agent", self.user_agent())
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let listing: SubredditListing = response.json().await?;
        Ok(listing)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedditUser {
    pub id: String,
    pub name: String,
    pub link_karma: i64,
    pub comment_karma: i64,
    pub created_utc: f64,
    pub is_gold: Option<bool>,
    pub is_mod: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct SubmitPost {
    pub title: String,
    pub kind: String,
    pub text: Option<String>,
    pub url: Option<String>,
    pub flair_id: Option<String>,
    pub nsfw: bool,
    pub spoiler: bool,
}

impl SubmitPost {
    pub fn text_post(title: &str, text: &str) -> Self {
        Self {
            title: title.to_string(),
            kind: "self".to_string(),
            text: Some(text.to_string()),
            url: None,
            flair_id: None,
            nsfw: false,
            spoiler: false,
        }
    }

    pub fn link_post(title: &str, url: &str) -> Self {
        Self {
            title: title.to_string(),
            kind: "link".to_string(),
            text: None,
            url: Some(url.to_string()),
            flair_id: None,
            nsfw: false,
            spoiler: false,
        }
    }

    pub fn flair(mut self, flair_id: &str) -> Self {
        self.flair_id = Some(flair_id.to_string());
        self
    }

    pub fn nsfw(mut self) -> Self {
        self.nsfw = true;
        self
    }

    pub fn spoiler(mut self) -> Self {
        self.spoiler = true;
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubmitResponse {
    pub url: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Deserialize)]
struct RedditSubmitWrapper {
    json: RedditSubmitJson,
}

#[derive(Deserialize)]
struct RedditSubmitJson {
    data: SubmitResponse,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommentResponse {
    pub things: Option<Vec<serde_json::Value>>,
}

#[derive(Deserialize)]
struct RedditCommentWrapper {
    json: RedditCommentJson,
}

#[derive(Deserialize)]
struct RedditCommentJson {
    data: CommentResponse,
}

pub enum VoteDirection {
    Up,
    Down,
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubredditListing {
    pub kind: String,
    pub data: ListingData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListingData {
    pub children: Vec<PostWrapper>,
    pub after: Option<String>,
    pub before: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostWrapper {
    pub kind: String,
    pub data: RedditPost,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedditPost {
    pub id: String,
    pub name: String,
    pub title: String,
    pub author: String,
    pub subreddit: String,
    pub selftext: Option<String>,
    pub url: Option<String>,
    pub score: i64,
    pub num_comments: u64,
    pub created_utc: f64,
    pub permalink: String,
    pub is_self: bool,
    pub over_18: bool,
    pub spoiler: bool,
}

#[derive(Default)]
pub struct SearchOptions {
    pub subreddit: Option<String>,
    pub sort: Option<String>,
    pub time: Option<String>,
    pub limit: Option<u32>,
}
