use ethers::{abi::Abi, prelude::*};
use std::sync::Arc;
use anyhow::Result;
use crate::tokens::TokenPair;

pub async fn get_price_from_dex(
    client: Arc<Provider<Http>>,
    router_address: &str,
    token_pair: &TokenPair,
) -> Result<f64> {
    let router: Address = router_address.parse()?;
    let abi_str = include_str!("../abis/uniswap_abi.json");
    let abi_uniswap: Abi = serde_json::from_str(abi_str)?;
    let contract = Contract::new(router, abi_uniswap, client);
    
    let path = vec![token_pair.base_token, token_pair.quote_token];
    let amount_in = U256::exp10(token_pair.base_decimals.into()); // Convert u8 to usize

    let result: Vec<U256> = contract
        .method::<_, Vec<U256>>("getAmountsOut", (amount_in, path))?
        .call()
        .await?;

    let quote_amount = result[1].as_u128() as f64 / 10_f64.powi(token_pair.quote_decimals as i32);
    Ok(quote_amount)
}

pub struct DexPrice {
    pub dex_name: String,
    pub price: f64,
    pub token_pair: String,
}

pub async fn get_prices_for_all_pairs(
    client: Arc<Provider<Http>>,
    router_address: &str,
    dex_name: &str,
    token_pairs: &[TokenPair],
) -> Vec<DexPrice> {
    let mut prices = Vec::new();
    
    for pair in token_pairs {
        match get_price_from_dex(client.clone(), router_address, pair).await {
            Ok(price) => {
                prices.push(DexPrice {
                    dex_name: dex_name.to_string(),
                    price,
                    token_pair: pair.name.clone(),
                });
            }
            Err(e) => {
                println!("❌ Failed to get price for {} from {}: {:?}", pair.name, dex_name, e);
            }
        }
    }
    
    prices
}
