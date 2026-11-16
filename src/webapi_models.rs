use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Deserializes JSON `null` as `T::default()` (Highrise often emits `null` for empty lists).
fn null_default<'de, D, T>(deserializer: D) -> std::result::Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutfitItemColors {
    #[serde(default)]
    pub active_palette: Option<i32>,
    #[serde(default)]
    pub palettes: Vec<Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutfitItem {
    #[serde(default)]
    pub item_id: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub colors: Option<OutfitItemColors>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveRoomInfo {
    #[serde(default)]
    pub room_id: Option<String>,
    #[serde(default)]
    pub disp_name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Crew {
    #[serde(default)]
    pub crew_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    #[serde(default, deserialize_with = "null_default")]
    pub user_id: String,
    #[serde(default, deserialize_with = "null_default")]
    pub username: String,
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(default)]
    pub discord_id: Option<String>,
    #[serde(default, deserialize_with = "null_default")]
    pub outfit: Vec<OutfitItem>,
    #[serde(default)]
    pub active_room: Option<ActiveRoomInfo>,
    #[serde(default)]
    pub crew: Option<Crew>,
    #[serde(default)]
    pub num_followers: Option<u64>,
    #[serde(default)]
    pub num_following: Option<u64>,
    #[serde(default)]
    pub num_friends: Option<u64>,
    #[serde(default)]
    pub joined_at: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomBasic {
    #[serde(default)]
    pub room_id: String,
    #[serde(default)]
    pub disp_name: String,
    #[serde(default)]
    pub owner_id: Option<String>,
    #[serde(default)]
    pub num_connected: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Room {
    #[serde(default)]
    pub room_id: String,
    #[serde(default)]
    pub disp_name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub owner_id: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub num_connected: Option<u64>,
    #[serde(default)]
    pub is_home_room: Option<bool>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub banner_url: Option<String>,
    #[serde(default)]
    pub crew_id: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    #[serde(default)]
    pub comment_id: Option<String>,
    #[serde(default)]
    pub author_id: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostItem {
    #[serde(default)]
    pub item_id: Option<String>,
    #[serde(default)]
    pub item_name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostInventory {
    #[serde(default)]
    pub items: Vec<PostItem>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostBody {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub inventory: Option<PostInventory>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostBasic {
    #[serde(default)]
    pub post_id: String,
    #[serde(default)]
    pub author_id: Option<String>,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Post {
    #[serde(default)]
    pub post_id: String,
    #[serde(default)]
    pub author_id: Option<String>,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub body: Option<PostBody>,
    #[serde(default)]
    pub num_likes: Option<u64>,
    #[serde(default)]
    pub num_comments: Option<u64>,
    #[serde(default)]
    pub comments: Vec<Comment>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkinPart {
    #[serde(rename = "imageFile", default)]
    pub image_file: Option<String>,
    #[serde(rename = "attachmentName", default)]
    pub attachment_name: Option<String>,
    #[serde(rename = "hasRemoteRenderLayer", default)]
    pub has_remote_render_layer: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Affiliation {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub creator_id: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelatedItem {
    #[serde(default)]
    pub item_id: String,
    #[serde(default)]
    pub item_name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelatedItems {
    #[serde(default)]
    pub items: Vec<RelatedItem>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seller {
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorefrontListings {
    #[serde(default)]
    pub listings: Vec<Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NFITemplateMetadata {
    #[serde(default)]
    pub template_id: Option<String>,
    #[serde(default)]
    pub max_supply: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NFIItemMetadata {
    #[serde(default)]
    pub token_id: Option<String>,
    #[serde(default)]
    pub template: Option<NFITemplateMetadata>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemMetadata {
    #[serde(default)]
    pub nfi: Option<NFIItemMetadata>,
    #[serde(default)]
    pub extra: Option<Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemBasic {
    #[serde(default)]
    pub item_id: String,
    #[serde(default)]
    pub item_name: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub rarity: Option<String>,
    #[serde(default)]
    pub is_purchasable: Option<bool>,
    #[serde(default)]
    pub is_tradable: Option<bool>,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    #[serde(default)]
    pub item_id: String,
    #[serde(default)]
    pub item_name: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub rarity: Option<String>,
    #[serde(default)]
    pub description_key: Option<String>,
    #[serde(default)]
    pub gems_sale_price: Option<u64>,
    #[serde(default)]
    pub pops_sale_price: Option<u64>,
    #[serde(default)]
    pub is_purchasable: bool,
    #[serde(default)]
    pub is_tradable: bool,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub link_ids: Vec<String>,
    #[serde(default)]
    pub m_front_skin_part_list: Vec<SkinPart>,
    #[serde(default)]
    pub m_back_skin_part_list: Vec<SkinPart>,
    #[serde(default)]
    pub affiliation: Option<Affiliation>,
    #[serde(default)]
    pub related_items: Option<RelatedItems>,
    #[serde(default)]
    pub seller: Option<Seller>,
    #[serde(default)]
    pub storefront_listings: Option<StorefrontListings>,
    #[serde(default)]
    pub metadata: Option<ItemMetadata>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reward {
    #[serde(default)]
    pub item_id: Option<String>,
    #[serde(default)]
    pub amount: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LimitedKompuReward {
    #[serde(default)]
    pub reward: Option<Reward>,
    #[serde(default)]
    pub remaining: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressReward {
    #[serde(default)]
    pub threshold: Option<u64>,
    #[serde(default)]
    pub reward: Option<Reward>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grab {
    #[serde(default)]
    pub grab_id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub rewards: Vec<Reward>,
    #[serde(default)]
    pub limited_rewards: Vec<LimitedKompuReward>,
    #[serde(default)]
    pub progress_rewards: Vec<ProgressReward>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SendMessageRequest {
    pub content: String,
    #[serde(rename = "type")]
    pub msg_type: &'static str,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetUserResponse {
    pub user: User,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetUsersResponse {
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub first_id: Option<String>,
    #[serde(default)]
    pub last_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetRoomResponse {
    pub room: Room,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetRoomsResponse {
    #[serde(default)]
    pub rooms: Vec<RoomBasic>,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub first_id: Option<String>,
    #[serde(default)]
    pub last_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetPostResponse {
    pub post: Post,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetPostsResponse {
    #[serde(default)]
    pub posts: Vec<PostBasic>,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub first_id: Option<String>,
    #[serde(default)]
    pub last_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetItemResponse {
    pub item: Item,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetItemsResponse {
    #[serde(default)]
    pub items: Vec<ItemBasic>,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub first_id: Option<String>,
    #[serde(default)]
    pub last_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchItemsResponse {
    #[serde(default)]
    pub items: Vec<Item>,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub first_id: Option<String>,
    #[serde(default)]
    pub last_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetGrabResponse {
    pub grab: Grab,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetGrabsResponse {
    #[serde(default)]
    pub grabs: Vec<Grab>,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub first_id: Option<String>,
    #[serde(default)]
    pub last_id: Option<String>,
}
