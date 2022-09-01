use std::future::Future;
use std::pin::Pin;
use std::process::Output;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::DatabaseTransaction;
use blockbuster::instruction::InstructionBundle;
use blockbuster::programs::bubblegum::BubblegumInstruction;
use digital_asset_types::dao::asset;
use crate::IngesterError;
use crate::program_transformers::bubblegum::update_asset;
use crate::program_transformers::common::save_changelog_event;

pub fn delegate<'c>(parsing_result: &BubblegumInstruction, bundle: &InstructionBundle, txn: &DatabaseTransaction) -> Pin<Box<dyn Future<Output=Result<_, IngesterError>> + Send + 'c>> {
    Box::pin(async move {
        if let (Some(le), Some(cl)) = (&parsing_result.leaf_update, &parsing_result.tree_update) {
            let seq = save_changelog_event(&cl, bundle.slot, txn)
                .await?
                .ok_or(IngesterError::ChangeLogEventMalformed)?;
            match le.schema {
                LeafSchema::V1 {
                    id,
                    delegate,
                    owner,
                    ..
                } => {
                    let id_bytes = id.to_bytes().to_vec();
                    let delegate = if owner == delegate {
                        None
                    } else {
                        Some(delegate.to_bytes().to_vec())
                    };
                    let owner_bytes = owner.to_bytes().to_vec();
                    let asset_to_update = asset::ActiveModel {
                        id: Unchanged(id_bytes.clone()),
                        leaf: Set(Some(le.leaf_hash.to_vec())),
                        delegate: Set(delegate),
                        owner: Set(owner_bytes),
                        seq: Set(seq as i64), // gummyroll seq
                        ..Default::default()
                    };
                    update_asset(txn, id_bytes, Some(seq), asset_to_update).await
                }
                _ => Err(IngesterError::NotImplemented),
            }
        }
        Err(IngesterError::ParsingError("Ix not parsed correctly".to_string()))
    })
}

