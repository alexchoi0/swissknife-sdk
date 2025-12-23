use rmcp::tool_box;
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "social")]
use swissknife_social_sdk as social;

#[derive(Clone)]
pub struct SocialTools {
    #[cfg(feature = "twitter")]
    pub twitter: Option<social::twitter::TwitterClient>,
    #[cfg(feature = "linkedin")]
    pub linkedin: Option<social::linkedin::LinkedInClient>,
    #[cfg(feature = "reddit")]
    pub reddit: Option<social::reddit::RedditClient>,
    #[cfg(feature = "youtube")]
    pub youtube: Option<social::youtube::YouTubeClient>,
}

impl SocialTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "twitter")]
            twitter: None,
            #[cfg(feature = "linkedin")]
            linkedin: None,
            #[cfg(feature = "reddit")]
            reddit: None,
            #[cfg(feature = "youtube")]
            youtube: None,
        }
    }

    #[cfg(feature = "twitter")]
    pub fn with_twitter(mut self, client: social::twitter::TwitterClient) -> Self {
        self.twitter = Some(client);
        self
    }

    #[cfg(feature = "linkedin")]
    pub fn with_linkedin(mut self, client: social::linkedin::LinkedInClient) -> Self {
        self.linkedin = Some(client);
        self
    }

    #[cfg(feature = "reddit")]
    pub fn with_reddit(mut self, client: social::reddit::RedditClient) -> Self {
        self.reddit = Some(client);
        self
    }

    #[cfg(feature = "youtube")]
    pub fn with_youtube(mut self, client: social::youtube::YouTubeClient) -> Self {
        self.youtube = Some(client);
        self
    }
}

impl Default for SocialTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TwitterPostTweetRequest {
    pub text: String,
    #[serde(default)]
    pub reply_to: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TwitterGetTweetRequest {
    pub tweet_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TwitterSearchRequest {
    pub query: String,
    #[serde(default)]
    pub max_results: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TwitterGetUserRequest {
    pub username: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LinkedInPostRequest {
    pub text: String,
    #[serde(default)]
    pub visibility: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LinkedInGetProfileRequest {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RedditGetPostRequest {
    pub subreddit: String,
    pub post_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RedditListPostsRequest {
    pub subreddit: String,
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RedditSubmitPostRequest {
    pub subreddit: String,
    pub title: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct YouTubeSearchRequest {
    pub query: String,
    #[serde(default)]
    pub max_results: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct YouTubeGetVideoRequest {
    pub video_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct YouTubeGetChannelRequest {
    pub channel_id: String,
}

#[tool_box]
impl SocialTools {
    #[cfg(feature = "twitter")]
    #[rmcp::tool(description = "Post a tweet on Twitter/X")]
    pub async fn twitter_post_tweet(
        &self,
        #[rmcp::tool(aggr)] req: TwitterPostTweetRequest,
    ) -> Result<String, String> {
        let client = self.twitter.as_ref()
            .ok_or_else(|| "Twitter client not configured".to_string())?;

        let tweet = client.create_tweet(&req.text, req.reply_to.as_deref()).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&tweet).map_err(|e| e.to_string())
    }

    #[cfg(feature = "twitter")]
    #[rmcp::tool(description = "Get a tweet by ID")]
    pub async fn twitter_get_tweet(
        &self,
        #[rmcp::tool(aggr)] req: TwitterGetTweetRequest,
    ) -> Result<String, String> {
        let client = self.twitter.as_ref()
            .ok_or_else(|| "Twitter client not configured".to_string())?;

        let tweet = client.get_tweet(&req.tweet_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&tweet).map_err(|e| e.to_string())
    }

    #[cfg(feature = "twitter")]
    #[rmcp::tool(description = "Search for tweets on Twitter/X")]
    pub async fn twitter_search(
        &self,
        #[rmcp::tool(aggr)] req: TwitterSearchRequest,
    ) -> Result<String, String> {
        let client = self.twitter.as_ref()
            .ok_or_else(|| "Twitter client not configured".to_string())?;

        let results = client.search_tweets(&req.query, req.max_results).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&results).map_err(|e| e.to_string())
    }

    #[cfg(feature = "twitter")]
    #[rmcp::tool(description = "Get a Twitter user's profile by username")]
    pub async fn twitter_get_user(
        &self,
        #[rmcp::tool(aggr)] req: TwitterGetUserRequest,
    ) -> Result<String, String> {
        let client = self.twitter.as_ref()
            .ok_or_else(|| "Twitter client not configured".to_string())?;

        let user = client.get_user_by_username(&req.username).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&user).map_err(|e| e.to_string())
    }

    #[cfg(feature = "linkedin")]
    #[rmcp::tool(description = "Create a post on LinkedIn")]
    pub async fn linkedin_post(
        &self,
        #[rmcp::tool(aggr)] req: LinkedInPostRequest,
    ) -> Result<String, String> {
        let client = self.linkedin.as_ref()
            .ok_or_else(|| "LinkedIn client not configured".to_string())?;

        let post = client.create_post(&req.text, req.visibility.as_deref()).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&post).map_err(|e| e.to_string())
    }

    #[cfg(feature = "linkedin")]
    #[rmcp::tool(description = "Get the authenticated user's LinkedIn profile")]
    pub async fn linkedin_get_profile(
        &self,
        #[rmcp::tool(aggr)] _req: LinkedInGetProfileRequest,
    ) -> Result<String, String> {
        let client = self.linkedin.as_ref()
            .ok_or_else(|| "LinkedIn client not configured".to_string())?;

        let profile = client.get_profile().await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&profile).map_err(|e| e.to_string())
    }

    #[cfg(feature = "reddit")]
    #[rmcp::tool(description = "Get a Reddit post by ID")]
    pub async fn reddit_get_post(
        &self,
        #[rmcp::tool(aggr)] req: RedditGetPostRequest,
    ) -> Result<String, String> {
        let client = self.reddit.as_ref()
            .ok_or_else(|| "Reddit client not configured".to_string())?;

        let post = client.get_post(&req.subreddit, &req.post_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&post).map_err(|e| e.to_string())
    }

    #[cfg(feature = "reddit")]
    #[rmcp::tool(description = "List posts from a subreddit")]
    pub async fn reddit_list_posts(
        &self,
        #[rmcp::tool(aggr)] req: RedditListPostsRequest,
    ) -> Result<String, String> {
        let client = self.reddit.as_ref()
            .ok_or_else(|| "Reddit client not configured".to_string())?;

        let posts = client.list_posts(
            &req.subreddit,
            req.sort.as_deref(),
            req.limit,
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&posts).map_err(|e| e.to_string())
    }

    #[cfg(feature = "reddit")]
    #[rmcp::tool(description = "Submit a new post to a subreddit")]
    pub async fn reddit_submit_post(
        &self,
        #[rmcp::tool(aggr)] req: RedditSubmitPostRequest,
    ) -> Result<String, String> {
        let client = self.reddit.as_ref()
            .ok_or_else(|| "Reddit client not configured".to_string())?;

        let post = client.submit_post(
            &req.subreddit,
            &req.title,
            req.text.as_deref(),
            req.url.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&post).map_err(|e| e.to_string())
    }

    #[cfg(feature = "youtube")]
    #[rmcp::tool(description = "Search for YouTube videos")]
    pub async fn youtube_search(
        &self,
        #[rmcp::tool(aggr)] req: YouTubeSearchRequest,
    ) -> Result<String, String> {
        let client = self.youtube.as_ref()
            .ok_or_else(|| "YouTube client not configured".to_string())?;

        let results = client.search(&req.query, req.max_results).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&results).map_err(|e| e.to_string())
    }

    #[cfg(feature = "youtube")]
    #[rmcp::tool(description = "Get details of a YouTube video")]
    pub async fn youtube_get_video(
        &self,
        #[rmcp::tool(aggr)] req: YouTubeGetVideoRequest,
    ) -> Result<String, String> {
        let client = self.youtube.as_ref()
            .ok_or_else(|| "YouTube client not configured".to_string())?;

        let video = client.get_video(&req.video_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&video).map_err(|e| e.to_string())
    }

    #[cfg(feature = "youtube")]
    #[rmcp::tool(description = "Get details of a YouTube channel")]
    pub async fn youtube_get_channel(
        &self,
        #[rmcp::tool(aggr)] req: YouTubeGetChannelRequest,
    ) -> Result<String, String> {
        let client = self.youtube.as_ref()
            .ok_or_else(|| "YouTube client not configured".to_string())?;

        let channel = client.get_channel(&req.channel_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&channel).map_err(|e| e.to_string())
    }
}
