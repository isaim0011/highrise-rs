use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::types::OutfitItem;

/// Live upstream free/untradable Highrise item IDs (`Djxhknz/Ids-highrise`).
pub const DEFAULT_ITEMS_URL: &str =
    "https://raw.githubusercontent.com/Djxhknz/Ids-highrise/main/untradable_items_new.json";

/// Clothing/avatar item entry from `Ids-highrise`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogItem {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub rarity: String,
    #[serde(default)]
    pub category: String,
}

impl CatalogItem {
    /// Converts this catalog item into a bot `OutfitItem` ready for `api.set_outfit(...)`.
    pub fn to_outfit_item(&self, active_palette: i32) -> OutfitItem {
        OutfitItem {
            item_type: "clothing".to_string(),
            id: self.id.clone(),
            amount: 1,
            account_bound: false,
            active_palette,
        }
    }
}

/// Searchable Highrise avatar/clothing catalog synced from GitHub or JSON.
#[derive(Debug, Clone, Default)]
pub struct ItemCatalog {
    items: Vec<CatalogItem>,
}

impl ItemCatalog {
    pub fn from_json(json_str: &str) -> Result<Self> {
        Ok(Self {
            items: serde_json::from_str(json_str)?,
        })
    }

    /// Fetches the latest `untradable_items_new.json` catalog from `Djxhknz/Ids-highrise`.
    pub async fn sync_from_github() -> Result<Self> {
        let text = reqwest::get(DEFAULT_ITEMS_URL)
            .await?
            .error_for_status()?
            .text()
            .await?;
        Self::from_json(&text)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Finds items whose `id` or `name` matches `query` (case-insensitive).
    pub fn search(&self, query: &str) -> Vec<&CatalogItem> {
        let q = query.trim().to_ascii_lowercase();
        self.items
            .iter()
            .filter(|it| {
                it.id.to_ascii_lowercase().contains(&q)
                    || it.name.to_ascii_lowercase().contains(&q)
            })
            .collect()
    }

    /// Lists items matching a category (e.g. `aura`, `shirt`, `hair_front`, `pants`).
    pub fn by_category(&self, category: &str) -> Vec<&CatalogItem> {
        self.items
            .iter()
            .filter(|it| it.category.eq_ignore_ascii_case(category.trim()))
            .collect()
    }
}
