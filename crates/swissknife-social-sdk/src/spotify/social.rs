use crate::{Error, Result};
use crate::spotify::SpotifyClient;
use serde::{Deserialize, Serialize};

impl SpotifyClient {
    pub async fn get_current_user(&self) -> Result<SpotifyUser> {
        let response = self.client()
            .get(format!("{}/me", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let user: SpotifyUser = response.json().await?;
        Ok(user)
    }

    pub async fn search(&self, query: &str, types: &[&str], limit: Option<u32>) -> Result<SearchResults> {
        let type_str = types.join(",");
        let limit = limit.unwrap_or(20);

        let response = self.client()
            .get(format!("{}/search", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&[
                ("q", query),
                ("type", &type_str),
                ("limit", &limit.to_string()),
            ])
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

        let results: SearchResults = response.json().await?;
        Ok(results)
    }

    pub async fn get_track(&self, track_id: &str) -> Result<Track> {
        let response = self.client()
            .get(format!("{}/tracks/{}", self.base_url(), track_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let track: Track = response.json().await?;
        Ok(track)
    }

    pub async fn get_album(&self, album_id: &str) -> Result<Album> {
        let response = self.client()
            .get(format!("{}/albums/{}", self.base_url(), album_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let album: Album = response.json().await?;
        Ok(album)
    }

    pub async fn get_artist(&self, artist_id: &str) -> Result<Artist> {
        let response = self.client()
            .get(format!("{}/artists/{}", self.base_url(), artist_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let artist: Artist = response.json().await?;
        Ok(artist)
    }

    pub async fn get_playlist(&self, playlist_id: &str) -> Result<Playlist> {
        let response = self.client()
            .get(format!("{}/playlists/{}", self.base_url(), playlist_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let playlist: Playlist = response.json().await?;
        Ok(playlist)
    }

    pub async fn get_user_playlists(&self, limit: Option<u32>) -> Result<PlaylistsResponse> {
        let limit = limit.unwrap_or(20);

        let response = self.client()
            .get(format!("{}/me/playlists", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let playlists: PlaylistsResponse = response.json().await?;
        Ok(playlists)
    }

    pub async fn create_playlist(&self, name: &str, description: Option<&str>, public: bool) -> Result<Playlist> {
        let user = self.get_current_user().await?;

        let body = serde_json::json!({
            "name": name,
            "description": description.unwrap_or(""),
            "public": public
        });

        let response = self.client()
            .post(format!("{}/users/{}/playlists", self.base_url(), user.id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .json(&body)
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

        let playlist: Playlist = response.json().await?;
        Ok(playlist)
    }

    pub async fn add_to_playlist(&self, playlist_id: &str, track_uris: Vec<String>) -> Result<()> {
        let body = serde_json::json!({
            "uris": track_uris
        });

        let response = self.client()
            .post(format!("{}/playlists/{}/tracks", self.base_url(), playlist_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .json(&body)
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

    pub async fn get_currently_playing(&self) -> Result<Option<CurrentlyPlaying>> {
        let response = self.client()
            .get(format!("{}/me/player/currently-playing", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .send()
            .await?;

        if response.status().as_u16() == 204 {
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let playing: CurrentlyPlaying = response.json().await?;
        Ok(Some(playing))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpotifyUser {
    pub id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub followers: Option<Followers>,
    pub images: Option<Vec<SpotifyImage>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Followers {
    pub total: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpotifyImage {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResults {
    pub tracks: Option<TracksResponse>,
    pub albums: Option<AlbumsResponse>,
    pub artists: Option<ArtistsResponse>,
    pub playlists: Option<PlaylistsResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TracksResponse {
    pub items: Vec<Track>,
    pub total: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlbumsResponse {
    pub items: Vec<Album>,
    pub total: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArtistsResponse {
    pub items: Vec<Artist>,
    pub total: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistsResponse {
    pub items: Vec<Playlist>,
    pub total: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub uri: String,
    pub duration_ms: u64,
    pub artists: Vec<ArtistSimple>,
    pub album: Option<AlbumSimple>,
    pub popularity: Option<u32>,
    pub preview_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArtistSimple {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlbumSimple {
    pub id: String,
    pub name: String,
    pub images: Option<Vec<SpotifyImage>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub artists: Vec<ArtistSimple>,
    pub images: Option<Vec<SpotifyImage>>,
    pub release_date: Option<String>,
    pub total_tracks: Option<u32>,
    pub tracks: Option<AlbumTracks>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlbumTracks {
    pub items: Vec<TrackSimple>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackSimple {
    pub id: String,
    pub name: String,
    pub duration_ms: u64,
    pub track_number: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub genres: Option<Vec<String>>,
    pub followers: Option<Followers>,
    pub popularity: Option<u32>,
    pub images: Option<Vec<SpotifyImage>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner: PlaylistOwner,
    pub public: Option<bool>,
    pub images: Option<Vec<SpotifyImage>>,
    pub tracks: Option<PlaylistTracks>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistOwner {
    pub id: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistTracks {
    pub total: u64,
    pub items: Option<Vec<PlaylistTrack>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistTrack {
    pub track: Option<Track>,
    pub added_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CurrentlyPlaying {
    pub is_playing: bool,
    pub item: Option<Track>,
    pub progress_ms: Option<u64>,
}
