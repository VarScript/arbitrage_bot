mod dex;

use dex::get_price_from_dex;
use dotenv::dotenv;
use ethers::prelude::*;
use std::{collections::HashMap, env, sync::Arc};
use colored::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    println!("\n{}", "⚡ ETHEREUM ARBITRAGE BOT ⚡".bright_blue().bold());
    println!("{}", "--------------------------------".bright_blue());

    let alchemy_key = env::var("ALCHEMY_API_KEY")?;
    let provider_url = format!("https://eth-mainnet.alchemyapi.io/v2/{}", alchemy_key);
    let provider = Provider::<Http>::try_from(provider_url)?;
    let client = Arc::new(provider);
    let gas_price = client.get_gas_price().await?;

    let gas_price_gwei = gas_price.as_u128() as f64 / 1e9;
    println!("\n    ⛽  GAS TRACKER");
    println!("    ├─ Current: {:.2} Gwei", gas_price_gwei);

    let gas_limit = 200_000;
    let eth_gas_cost: U256 = gas_price * gas_limit;
    let eth_gas_cost_f64 = eth_gas_cost.as_u128() as f64 / 1e18;
    println!("    ├─ Estimated TX Cost: {:.6} ETH", eth_gas_cost_f64);

    // Add PancakeSwap
    let routers = vec![
        ("Uniswap", "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"),
        ("SushiSwap", "0xd9e1cE17f2641f24aE83637ab66a2cca9C378B9F"),
        ("PancakeSwap", "0x1097053Fd2ea711dad45caCcc45EfF7548fCB362"),
    ];

    println!("\n    📊  PRICE DISCOVERY");
    let mut prices: HashMap<&str, f64> = HashMap::new();
    let mut best = ("", 0.0);
    let mut worst = ("", f64::MAX);

    for (name, router) in &routers {
        match get_price_from_dex(client.clone(), router).await {
            Ok(price) => {
                prices.insert(name, price);
                if price > best.1 {
                    best = (name, price);
                }
                if price < worst.1 {
                    worst = (name, price);
                }
            }
            Err(e) => {
                println!("❌ Failed to get price from {}: {:?}", name, e);
            }
        }
    }

    for (dex, price) in &prices {
        let tag = if *dex == best.0 {
            "🟢 (Best)"
        } else if *dex == worst.0 {
            "🔴 (Worst)"
        } else {
            " "
        };
        println!("    ├─ {:11}: {:>10.6} USDC  {}", dex, price, tag);
    }

    let spread = best.1 - worst.1;
    println!(
        "    └─ Spread:     ████████ {:.2} USDC  📈",
        spread
    );

    let gas_cost_usdc = eth_gas_cost_f64 * best.1;
    let net_profit = spread - gas_cost_usdc;
    let roi = (net_profit / (worst.1)) * 100.0;

    println!("\n    💰  PROFIT ANALYSIS");
    println!("    ├─ Gross Profit:  {:.2} USDC", spread);
    println!("    ├─ Net Profit:    {:.2} USDC", net_profit);
    println!("    └─ ROI:           {:.2}%  📊", roi);

    let threshold = 2.0;
    println!("\n    VERDICT: {}",
        if net_profit > threshold {
            format!("✅ EXECUTE ${:.2}  (Threshold: ${})", net_profit, threshold).green()
        } else {
            format!("❌ SKIP ${:.2}  (Below Threshold)", net_profit).red()
        }
    );

    Ok(())
}
