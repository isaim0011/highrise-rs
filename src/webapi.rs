use reqwest::{Client as HttpClient, Response, StatusCode};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::webapi_models::{
    GetGrabResponse, GetGrabsResponse, GetItemResponse, GetItemsResponse, GetPostResponse,
    GetPostsResponse, GetRoomResponse, GetRoomsResponse, GetUserResponse, GetUsersResponse,
    SearchItemsResponse, SendMessageRequest,
};

const DEFAULT_WEBAPI_URL: &str = "https://webapi.highrise.game";

/// Sort order for paginated Highrise Web API list endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    #[default]
    Desc,
    Asc,
}

impl SortOrder {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Desc => "desc",
            Self::Asc => "asc",
        }
    }
}

/// Shared cursor-based pagination parameters for Web API list calls.
#[derive(Debug, Clone, Default)]
pub struct PageQuery {
    pub starts_after: Option<String>,
    pub ends_before: Option<String>,
    pub sort_order: SortOrder,
    pub limit: Option<u32>,
}

impl PageQuery {
    fn push_params(&self, pairs: &mut Vec<(&'static str, String)>) {
        if let Some(v) = &self.starts_after {
            pairs.push(("starts_after", v.clone()));
        }
        if let Some(v) = &self.ends_before {
            pairs.push(("ends_before", v.clone()));
        }
        pairs.push(("sort_order", self.sort_order.as_str().to_string()));
        pairs.push(("limit", self.limit.unwrap_or(20).to_string()));
    }
}

/// Async client for Highrise's REST Web API (`https://webapi.highrise.game`).
#[derive(Clone)]
pub struct WebApi {
    base_url: String,
    http: HttpClient,
}

impl Default for WebApi {
    fn default() -> Self {
        Self::new()
    }
}

impl WebApi {
    pub fn new() -> Self {
        let base = std::env::var("HR_WEBAPI_URL").unwrap_or_else(|_| DEFAULT_WEBAPI_URL.to_string());
        Self::with_base_url(base)
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let mut builder = HttpClient::builder();
        if let Ok(ip_str) = std::env::var("BIND_SOURCE_IP") {
            if let Ok(ip) = ip_str.parse::<std::net::IpAddr>() {
                builder = builder.local_address(ip);
            }
        }
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            http: builder.build().unwrap_or_default(),
        }
    }

    async fn parse_response<T: serde::de::DeserializeOwned>(resp: Response) -> Result<T> {
        let status = resp.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());
            return Err(Error::RateLimited { retry_after });
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            let msg = serde_json::from_str::<Value>(&body)
                .ok()
                .and_then(|v| {
                    v.get("message")
                        .or_else(|| v.get("detail"))
                        .or_else(|| v.get("error"))
                        .and_then(|m| m.as_str())
                        .map(str::to_string)
                })
                .unwrap_or(body);
            return Err(Error::WebApi {
                status: status.as_u16(),
                message: msg,
            });
        }
        Ok(resp.json::<T>().await?)
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&'static str, String)],
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.http.get(url).query(query).send().await?;
        Self::parse_response(resp).await
    }

    pub async fn get_user(&self, user_id: &str) -> Result<GetUserResponse> {
        self.get_json(&format!("/users/{user_id}"), &[]).await
    }

    pub async fn get_users(&self, page: &PageQuery, username: Option<&str>) -> Result<GetUsersResponse> {
        let mut q = Vec::new();
        page.push_params(&mut q);
        if let Some(name) = username {
            q.push(("username", name.to_string()));
        }
        self.get_json("/users", &q).await
    }

    pub async fn get_room(&self, room_id: &str) -> Result<GetRoomResponse> {
        self.get_json(&format!("/rooms/{room_id}"), &[]).await
    }

    pub async fn get_rooms(
        &self,
        page: &PageQuery,
        room_name: Option<&str>,
        owner_id: Option<&str>,
    ) -> Result<GetRoomsResponse> {
        let mut q = Vec::new();
        page.push_params(&mut q);
        if let Some(name) = room_name {
            q.push(("room_name", name.to_string()));
        }
        if let Some(owner) = owner_id {
            q.push(("owner_id", owner.to_string()));
        }
        self.get_json("/rooms", &q).await
    }

    pub async fn get_post(&self, post_id: &str) -> Result<GetPostResponse> {
        self.get_json(&format!("/posts/{post_id}"), &[]).await
    }

    pub async fn get_posts(&self, page: &PageQuery, author_id: Option<&str>) -> Result<GetPostsResponse> {
        let mut q = Vec::new();
        page.push_params(&mut q);
        if let Some(author) = author_id {
            q.push(("author_id", author.to_string()));
        }
        self.get_json("/posts", &q).await
    }

    pub async fn get_item(&self, item_id: &str) -> Result<GetItemResponse> {
        self.get_json(&format!("/items/{item_id}"), &[]).await
    }

    pub async fn get_items(
        &self,
        page: &PageQuery,
        item_name: Option<&str>,
        category: Option<&str>,
        rarity: Option<&str>,
    ) -> Result<GetItemsResponse> {
        let mut q = Vec::new();
        page.push_params(&mut q);
        if let Some(name) = item_name {
            q.push(("item_name", name.to_string()));
        }
        if let Some(cat) = category {
            q.push(("category", cat.to_string()));
        }
        if let Some(rar) = rarity {
            q.push(("rarity", rar.to_string()));
        }
        self.get_json("/items", &q).await
    }

    /// Searches catalog items via `GET /items/search`.
    pub async fn search_items(
        &self,
        page: &PageQuery,
        query_text: &str,
        category: Option<&str>,
    ) -> Result<SearchItemsResponse> {
        let mut q = Vec::new();
        page.push_params(&mut q);
        q.push(("query", query_text.to_string()));
        if let Some(cat) = category {
            q.push(("category", cat.to_string()));
        }
        self.get_json("/items/search", &q).await
    }

    pub async fn get_grab(&self, grab_id: &str) -> Result<GetGrabResponse> {
        self.get_json(&format!("/grabs/{grab_id}"), &[]).await
    }

    pub async fn get_grabs(&self, page: &PageQuery, title: Option<&str>) -> Result<GetGrabsResponse> {
        let mut q = Vec::new();
        page.push_params(&mut q);
        if let Some(t) = title {
            q.push(("title", t.to_string()));
        }
        self.get_json("/grabs", &q).await
    }

    /// Sends a message to a conversation via `POST /conversations/{conversation_id}/messages`.
    pub async fn send_conversation_message(
        &self,
        conversation_id: &str,
        api_token: &str,
        content: impl Into<String>,
    ) -> Result<Value> {
        let url = format!("{}/conversations/{conversation_id}/messages", self.base_url);
        let body = SendMessageRequest {
            content: content.into(),
            msg_type: "text",
        };
        let resp = self
            .http
            .post(url)
            .header("api-token", api_token)
            .json(&body)
            .send()
            .await?;
        Self::parse_response(resp).await
    }
}
