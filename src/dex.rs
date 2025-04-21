use ethers::{abi::Abi, prelude::*};
use std::sync::Arc;
use anyhow::Result;

const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

pub async fn get_price_from_dex(
    client: Arc<Provider<Http>>,
    router_address: &str,
) -> Result<f64> {
    let router: Address = router_address.parse()?;
    let weth: Address = WETH.parse()?;
    let usdc: Address = USDC.parse()?;

    let abi_str = include_str!("../uniswap_abi.json");
    let abi_uniswap: Abi = serde_json::from_str(abi_str)?;
    let contract = Contract::new(router, abi_uniswap, client);
    let path = vec![weth, usdc];
    let amount_in = U256::exp10(18); // 1 ETH in wei

    let result: Vec<U256> = contract
        .method::<_, Vec<U256>>("getAmountsOut", (amount_in, path))?
        .call()
        .await?;

    let usdc_amount = result[1].as_u128() as f64 / 1_000_000.0;
    Ok(usdc_amount)
}
