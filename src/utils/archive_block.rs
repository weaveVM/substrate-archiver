use {
    crate::utils::{
        block_type::Block,
        env_var::get_env_var,
        get_block::{by_number, get_current_block_number},
        planetscale::{ps_archive_block, ps_get_latest_block_id},
        schema::Network,
        transaction::{send_wvm_calldata, send_wvm_calldata_backfill},
    },
    anyhow::Error,
    std::{thread, time::Duration},
};

pub async fn archive(block_number: Option<u64>, is_backfill: bool) -> Result<String, Error> {
    let network = Network::config();
    let block_to_archive = block_number.unwrap_or(if is_backfill {
        get_env_var("backfill_start_block")
            .unwrap()
            .parse::<u64>()
            .unwrap()
    } else {
        network.start_block
    });

    // assert to ensure backfill and livesync dont collide at the continuity checkpoint
    if is_backfill {
        assert!(block_number.unwrap() < network.start_block)
    }
    // fetch block
    let block_data_json = by_number(block_to_archive).await.unwrap();
    let block_data_str = serde_json::to_string(&block_data_json).unwrap();
    // serialize response into Block struct
    let block_data_struct = Block::new(block_data_str);
    // borsh serialize the struct
    let borsh_res = Block::borsh_ser(&block_data_struct);
    // brotli compress the borsh serialized block
    let brotli_res = Block::brotli_compress(&borsh_res);

    let txid = if is_backfill {
        send_wvm_calldata_backfill(brotli_res).await.unwrap()
    } else {
        send_wvm_calldata(brotli_res).await.unwrap()
    };

    Ok(txid)
}

pub async fn sprint_blocks_archiving(is_backfill: bool) {
    let network = Network::config();
    let block_time = network.block_time;
    let mut current_block_number = get_current_block_number().await;
    // it defaults to network.start_block or env.backfill_start_block
    // based on is_backfill if planestcale fails
    let mut start_block = ps_get_latest_block_id(is_backfill).await;

    loop {
        if start_block < current_block_number - 1 {
            println!("\n{}", "#".repeat(100));
            println!(
                "\nARCHIVING BLOCK #{} of Network {} -- IS_BACKFILL: {}\n",
                start_block, network.name, is_backfill
            );
            let archive_txid = archive(Some(start_block), is_backfill).await.unwrap();
            let _ = ps_archive_block(&start_block, &archive_txid, is_backfill).await;
            start_block += 1;
            println!("\n{}", "#".repeat(100));
        } else {
            current_block_number = get_current_block_number().await;
            thread::sleep(Duration::from_secs(block_time as u64));
        }
    }
}
