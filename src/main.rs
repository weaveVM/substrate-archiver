use crate::utils::archive_block::archive;
use crate::utils::schema::Network;
use crate::utils::planetscale::{ps_archive_block, ps_get_latest_block_id};
use std::thread;
use std::time::Duration;
use axum::{routing::get, Router};
use tokio::task;

mod utils;
#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let network = Network::config();
    let block_time = network.block_time;
    let ps_latest_archived_block = ps_get_latest_block_id().await;
    // it defaults to network.start_block if planestcale fails
    let mut start_block = ps_latest_archived_block;

    println!("\n{:#?}\n\n", network);

    let router = Router::new().route("/", get(weave_gm));

    // poll blocks & archive in parallel
    task::spawn(async move {
        loop {
            println!("\n{}", "#".repeat(100));
            println!(
                "\nARCHIVING BLOCK #{} of Network {} -- ChainId: {}\n",
                start_block, network.name, network.network_chain_id
            );
            let archive_txid = archive(Some(start_block)).await.unwrap();
            let _ = ps_archive_block(&start_block, &archive_txid).await;
            start_block += 1;
            println!("\n{}", "#".repeat(100));
            thread::sleep(Duration::from_secs(block_time.into()));
        }
    });

    Ok(router.into())
}


async fn weave_gm() -> &'static str {
    "WeaveGM!"
}

