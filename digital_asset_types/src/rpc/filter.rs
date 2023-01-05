use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AssetSorting {
pub sort_by: AssetSortBy,
pub sort_direction: AssetSortDirection,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum AssetSortBy {
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "updated")]
    Updated,
    #[serde(rename = "recent_action")]
    RecentAction,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum AssetSortDirection {
    Asc,
    Desc,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum OfferSorting {
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "updated")]
    Updated,
    #[serde(rename = "price")]
    Price,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum ListingSorting {
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "updated")]
    Updated,
    #[serde(rename = "price")]
    Price,
    #[serde(rename = "number_of_offers")]
    NumberOfOffers,
}
