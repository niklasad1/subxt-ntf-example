use std::marker::PhantomData;

use pallet_nfts::{CollectionConfig, CollectionSettings, ItemSettings, MintSettings, MintType};
use subxt::{utils::Static, OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::dev::{self};

#[subxt::subxt(
    runtime_metadata_path = "metadata.scale",
    substitute_type(
        path = "pallet_nfts::types::CollectionConfig<Price, BlockNumber, CollectionId>",
        with = "::subxt::utils::Static<::pallet_nfts::CollectionConfig<Price, BlockNumber, CollectionId>>"
    )
)]
pub mod assethub {}

/// Example of converting LockCollectionSettings to subxt::BitFlags1
fn to_lock_settings(
    settings: CollectionSettings,
) -> assethub::runtime_types::pallet_nfts::types::BitFlags1<
    assethub::runtime_types::pallet_nfts::types::CollectionSetting,
> {
    let bits = settings.0.bits();
    assethub::runtime_types::pallet_nfts::types::BitFlags1(bits, PhantomData)
}

async fn lock_collection(
    api: &OnlineClient<PolkadotConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Bonus: lock the collection
    //
    // use proper collection id.

    let payload = assethub::tx()
        .nfts()
        .lock_collection(0, to_lock_settings(CollectionSettings::all_enabled()));
    let from = dev::alice();
    let events = api
        .tx()
        .sign_and_submit_then_watch_default(&payload, &from)
        .await?
        .wait_for_finalized_success()
        .await?;

    tracing::info!("{:?}", events);

    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let api =
        OnlineClient::<PolkadotConfig>::from_url("wss://asset-hub-kusama-rpc.dwellir.com").await?;
    tracing::info!("Connection with parachain established.");

    let config = CollectionConfig {
        settings: CollectionSettings::all_enabled(),
        max_supply: None,
        mint_settings: MintSettings {
            mint_type: MintType::HolderOf(1),
            price: Some(100),
            start_block: None,
            end_block: Some(100),
            default_item_settings: ItemSettings::all_enabled(),
        },
    };

    // Create a collection
    // Note: we have replaced the CollectionConfig with pallet_nfts::CollectionConfig<Price, BlockNumber, CollectionId>
    // to use the proper types from polkadot-sdk
    //
    // It's also possible to write wrapper functions to convert the types into generated types that subxt requires
    // but it's not that nice to do so.
    //
    // See to_lock_settings function for an example of how to convert LockCollectionSettings to subxt::BitFlags1
    let admin = dev::bob().public_key().into();
    let payload = assethub::tx().nfts().create(admin, Static(config));
    let from = dev::alice();
    let events = api
        .tx()
        .sign_and_submit_then_watch_default(&payload, &from)
        .await?
        .wait_for_finalized_success()
        .await?;

    tracing::info!("{:?}", events);

    Ok(())
}
