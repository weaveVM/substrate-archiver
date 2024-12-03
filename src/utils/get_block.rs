use {crate::utils::schema::Network, anyhow::Error, serde_json, ureq};

pub async fn by_number(number: u64) -> Result<serde_json::Value, Error> {
    let network = Network::config();
    let req_url = format!("{}/blocks/{}", network.network_rpc, number);

    // Just return the JSON response directly
    let response = ureq::get(&req_url).call()?.into_string()?;

    Ok(serde_json::from_str(&response)?)
}

pub async fn get_current_block_number() -> u64 {
    let network: Network = Network::config();
    // connect to the target EVM provider
    let req_url = format!("{}/blocks/head", network.network_rpc);

    let req = ureq::get(&req_url)
        .call()
        .unwrap()
        .into_string()
        .unwrap()
        .parse::<serde_json::Value>()
        .unwrap();
    req["number"].as_str().unwrap().parse::<u64>().unwrap()
}
