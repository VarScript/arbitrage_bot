use dotenv::dotenv;
use ethers::abi::Abi;
use ethers::prelude::*;
use std::{
    collections::HashMap, env, sync::Arc,
};

const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let alchemy_key = env::var("ALCHEMY_API_KEY")?;
    let provider_url = format!("https://eth-mainnet.alchemyapi.io/v2/{}", alchemy_key);
    let provider = Provider::<Http>::try_from(provider_url)?;
    let client = Arc::new(provider);

    // Routers for Uniswap, SushiSwap, PancakeSwap
    let routers = vec![
        ("Uniswap", "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"),
        ("SushiSwap", "0xd9e1cE17f2641f24aE83637ab66a2cca9C378B9F"),
    ];

    let mut prices: HashMap<&str, f64> = HashMap::new();

    for (name, router) in routers.iter() {
        match get_price_from_dex(client.clone(), router).await {
            Ok(price) => {
                println!("{}: 1 ETH = {:.2} USDC", name, price);
                prices.insert(name, price);
            }
            Err(e) => {
                println!("Failed to get price from {}: {}", name, e);
            }
        };
    };

    // Find best price
    if let Some((dex, best_price)) = prices.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()) {
        println!(
            "\nBest price to SELL ETH: {} at {:.2} USDC",
            dex, best_price
        );
    }

    if let Some((dex, worst_price)) = prices.iter().min_by(|a, b| a.1.partial_cmp(b.1).unwrap()) {
        println!(
            "Worst price to SELL ETH: {} at {:.2} USDC",
            dex, worst_price
        );
    }

    Ok(())
}

async fn get_price_from_dex(
    client: Arc<Provider<Http>>,
    router_address: &str,
) -> anyhow::Result<f64> {
    let router: Address = router_address.parse()?;
    let weth: Address = WETH.parse()?;
    let usdc: Address = USDC.parse()?;

    let abi_str = include_str!("uniswap_abi.json");
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
