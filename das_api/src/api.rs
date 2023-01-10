use crate::{DasApiError, RpcModule};
use async_trait::async_trait;
use digital_asset_types::rpc::filter::{AssetSortBy, AssetSortDirection};
use digital_asset_types::rpc::response::AssetList;
use digital_asset_types::rpc::{Asset, AssetProof};
use digital_asset_types::{dapi::SearchAssetsQuery, rpc::filter::AssetSorting};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetAssetsByGroup {
    group_key: String,
    group_value: String,
    sort_by: Option<AssetSorting>,
    limit: Option<u32>,
    page: Option<u32>,
    before: Option<String>,
    after: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetAssetsByOwner {
    owner_address: String,
    sort_by: Option<AssetSorting>,
    limit: Option<u32>,
    page: Option<u32>,
    before: Option<String>,
    after: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetAsset {
    id: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]

struct GetAssetsByCreator {
    creator_address: String,
    sort_by: Option<AssetSorting>,
    limit: Option<u32>,
    page: Option<u32>,
    before: Option<String>,
    after: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]

struct SearchAssets {
    search_expression: SearchAssetsQuery,
    sort_by: Option<AssetSorting>,
    limit: Option<u32>,
    page: Option<u32>,
    before: Option<String>,
    after: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]

struct GetAssetsByAuthority {
    authority_address: String,
    sort_by: Option<AssetSorting>,
    limit: Option<u32>,
    page: Option<u32>,
    before: Option<String>,
    after: Option<String>,
}

#[async_trait]
pub trait ApiContract: Send + Sync + 'static {
    async fn check_health(&self) -> Result<(), DasApiError>;
    async fn get_asset_proof(&self, asset_id: String) -> Result<AssetProof, DasApiError>;
    async fn get_asset(&self, asset_id: String) -> Result<Asset, DasApiError>;
    async fn get_assets_by_owner(
        &self,
        owner_address: String,
        sort_by: AssetSorting,
        limit: Option<u32>,
        page: Option<u32>,
        before: Option<String>,
        after: Option<String>,
    ) -> Result<AssetList, DasApiError>;
    async fn get_assets_by_group(
        &self,
        group_key: String,
        group_value: String,
        sort_by: AssetSorting,
        limit: Option<u32>,
        page: Option<u32>,
        before: Option<String>,
        after: Option<String>,
    ) -> Result<AssetList, DasApiError>;
    async fn get_assets_by_creator(
        &self,
        creator_expression: Vec<String>,
        sort_by: AssetSorting,
        limit: Option<u32>,
        page: Option<u32>,
        before: Option<String>,
        after: Option<String>,
    ) -> Result<AssetList, DasApiError>;
    async fn get_assets_by_authority(
        &self,
        authority_expression: String,
        sort_by: AssetSorting,
        limit: Option<u32>,
        page: Option<u32>,
        before: Option<String>,
        after: Option<String>,
    ) -> Result<AssetList, DasApiError>;
    async fn search_assets(
        &self,
        search_expression: SearchAssetsQuery,
        sort_by: AssetSorting,
        limit: Option<u32>,
        page: Option<u32>,
        before: Option<String>,
        after: Option<String>,
    ) -> Result<AssetList, DasApiError>;
}

pub struct RpcApiBuilder;

impl RpcApiBuilder {
    pub fn build(
        contract: Box<dyn ApiContract>,
    ) -> Result<RpcModule<Box<dyn ApiContract>>, DasApiError> {
        let mut module = RpcModule::new(contract);
        module.register_async_method("healthz", |_rpc_params, rpc_context| async move {
            println!("Checking Health");
            rpc_context.check_health().await.map_err(Into::into)
        })?;

        module.register_async_method("get_asset_proof", |rpc_params, rpc_context| async move {
            let payload = rpc_params.parse::<GetAsset>();
            let asset_id = match payload {
                Ok(payload) => Ok(payload.id),
                Err(_) => rpc_params.one::<String>(),
            }?;
            println!("Asset Id {}", asset_id);
            rpc_context
                .get_asset_proof(asset_id)
                .await
                .map_err(Into::into)
        })?;
        module.register_alias("getAssetProof", "get_asset_proof")?;

        module.register_async_method("get_asset", |rpc_params, rpc_context| async move {
            let payload = rpc_params.parse::<GetAsset>();
            let asset_id = match payload {
                Ok(payload) => Ok(payload.id),
                Err(_) => rpc_params.one::<String>(),
            }?;
            println!("Asset Id {}", asset_id);
            rpc_context.get_asset(asset_id).await.map_err(Into::into)
        })?;
        module.register_alias("getAsset", "get_asset")?;

        module.register_async_method(
            "get_assets_by_owner",
            |rpc_params, rpc_context| async move {
                let payload = rpc_params.parse::<GetAssetsByOwner>();
                let (owner_address, sort_by, limit, page, before, after) = match payload {
                    Ok(payload) => {
                        let GetAssetsByOwner {
                            owner_address,
                            sort_by,
                            limit,
                            page,
                            before,
                            after,
                        } = payload;
                        (owner_address, sort_by, limit, page, before, after)
                    }
                    Err(e) => {
                        println!("Error parsing payload {:?}", e);
                        let (owner_address, sort_by, limit, page, before, after) = rpc_params
                            .parse::<(
                                String,
                                AssetSortBy,
                                Option<u32>,
                                Option<u32>,
                                Option<String>,
                                Option<String>,
                            )>()?;
                        let before: Option<String> = before.filter(|before| !before.is_empty());
                        let after: Option<String> = after.filter(|after| !after.is_empty());
                        (
                            owner_address,
                            Some(AssetSorting {
                                sort_by,
                                sort_direction: AssetSortDirection::Desc,
                            }),
                            limit,
                            page,
                            before,
                            after,
                        )
                    }
                };
                let sort_by = sort_by.unwrap_or_default();
                rpc_context
                    .get_assets_by_owner(owner_address, sort_by, limit, page, before, after)
                    .await
                    .map_err(Into::into)
            },
        )?;
        module.register_alias("getAssetsByOwner", "get_assets_by_owner")?;

        module.register_async_method(
            "get_assets_by_creator",
            |rpc_params, rpc_context| async move {
                let payload = rpc_params.parse::<GetAssetsByCreator>();
                let (creator_address, sort_by, limit, page, before, after) = match payload {
                    Ok(payload) => {
                        let GetAssetsByCreator {
                            creator_address,
                            sort_by,
                            limit,
                            page,
                            before,
                            after,
                        } = payload;
                        (creator_address, sort_by, limit, page, before, after)
                    }
                    Err(_) => {
                        let (creator_address, sort_by, limit, page, before, after) = rpc_params
                            .parse::<(
                                String,
                                AssetSortBy,
                                Option<u32>,
                                Option<u32>,
                                Option<String>,
                                Option<String>,
                            )>()?;
                        let before: Option<String> = before.filter(|before| !before.is_empty());
                        let after: Option<String> = after.filter(|after| !after.is_empty());
                        (
                            creator_address,
                            Some(AssetSorting {
                                sort_by,
                                sort_direction: AssetSortDirection::Desc,
                            }),
                            limit,
                            page,
                            before,
                            after,
                        )
                    }
                };
                let sort_by = sort_by.unwrap_or_default();
                rpc_context
                    .get_assets_by_creator(
                        vec![creator_address],
                        sort_by,
                        limit,
                        page,
                        before,
                        after,
                    )
                    .await
                    .map_err(Into::into)
            },
        )?;
        module.register_alias("getAssetsByCreator", "get_assets_by_creator")?;

        module.register_async_method(
            "getAssetsByAuthority",
            |rpc_params, rpc_context| async move {
                let GetAssetsByAuthority {
                    authority_address,
                    sort_by,
                    limit,
                    page,
                    before,
                    after,
                } = rpc_params.parse::<GetAssetsByAuthority>()?;
                let sort_by = sort_by.unwrap_or_default();
                rpc_context
                    .get_assets_by_authority(authority_address, sort_by, limit, page, before, after)
                    .await
                    .map_err(Into::into)
            },
        )?;

        module.register_async_method(
            "get_assets_by_group",
            |rpc_params, rpc_context| async move {
                let payload = rpc_params.parse::<GetAssetsByGroup>();
                let (group_key, group_value, sort_by, limit, page, before, after) = match payload {
                    Ok(payload) => {
                        let GetAssetsByGroup {
                            group_key,
                            group_value,
                            sort_by,
                            limit,
                            page,
                            before,
                            after,
                        } = payload;
                        (group_key, group_value, sort_by, limit, page, before, after)
                    }
                    Err(_) => {
                        let (group_key, group_value, sort_by, limit, page, before, after) =
                            rpc_params.parse::<(
                                String,
                                String,
                                AssetSortBy,
                                Option<u32>,
                                Option<u32>,
                                Option<String>,
                                Option<String>,
                            )>()?;
                        let before: Option<String> = before.filter(|before| !before.is_empty());
                        let after: Option<String> = after.filter(|after| !after.is_empty());
                        (
                            group_key,
                            group_value,
                            Some(AssetSorting {
                                sort_by,
                                sort_direction: AssetSortDirection::Desc,
                            }),
                            limit,
                            page,
                            before,
                            after,
                        )
                    }
                };

                let sort_by = sort_by.unwrap_or_default();
                rpc_context
                    .get_assets_by_group(
                        group_key,
                        group_value,
                        sort_by,
                        limit,
                        page,
                        before,
                        after,
                    )
                    .await
                    .map_err(Into::into)
            },
        )?;
        module.register_alias("getAssetsByGroup", "get_assets_by_group")?;

        module.register_async_method("search_assets", |rpc_params, rpc_context| async move {
            let SearchAssets {
                search_expression,
                sort_by,
                limit,
                page,
                before,
                after,
            } = rpc_params.parse::<SearchAssets>()?;
            let sort_by = sort_by.unwrap_or_default();
            rpc_context
                .search_assets(search_expression, sort_by, limit, page, before, after)
                .await
                .map_err(Into::into)
        })?;
        module.register_alias("searchAssets", "search_assets")?;

        Ok(module)
    }
}
