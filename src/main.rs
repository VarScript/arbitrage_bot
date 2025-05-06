mod dex;
mod tokens;
mod curve;

use dex::{get_prices_for_all_pairs as get_dex_prices, DexPrice};
use curve::get_prices_for_all_pairs as get_curve_prices;
use tokens::TokenPairs;
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

    let routers = vec![
        ("Uniswap", "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"),
        ("SushiSwap", "0xd9e1cE17f2641f24aE83637ab66a2cca9C378B9F"),
    ];

    let token_pairs = TokenPairs::new();
    let mut all_prices: Vec<DexPrice> = Vec::new();

    println!("\n    📊  PRICE DISCOVERY");
    
    // Get prices from DEXes
    for (name, router) in &routers {
        let prices = get_dex_prices(client.clone(), router, name, &token_pairs.pairs).await;
        all_prices.extend(prices);
    }

    // Get prices from Curve
    let curve_prices = get_curve_prices(client.clone(), &token_pairs.pairs).await;
    for price in curve_prices {
        all_prices.push(DexPrice {
            dex_name: format!("Curve {}", price.pool_name),
            price: price.price,
            token_pair: price.token_pair,
        });
    }

    // Group prices by token pair
    let mut pair_prices: HashMap<String, Vec<&DexPrice>> = HashMap::new();
    for price in &all_prices {
        pair_prices.entry(price.token_pair.clone())
            .or_insert_with(Vec::new)
            .push(price);
    }

    // Analyze each pair
    for (pair_name, prices) in pair_prices {
        println!("\n    {}:", pair_name.bright_yellow());
        
        let mut best = ("", 0.0);
        let mut worst = ("", f64::MAX);

        for price in &prices {
            if price.price > best.1 {
                best = (&price.dex_name, price.price);
            }
            if price.price < worst.1 {
                worst = (&price.dex_name, price.price);
            }
        }

        for price in &prices {
            let tag = if price.dex_name == best.0 {
                "🟢 (Best)"
            } else if price.dex_name == worst.0 {
                "🔴 (Worst)"
            } else {
                " "
            };
            println!("    ├─ {:11}: {:>10.6}  {}", price.dex_name, price.price, tag);
        }

        let spread = best.1 - worst.1;
        println!(
            "    └─ Spread:     ████████ {:.6}  📈",
            spread
        );

        let gas_cost_usdc = eth_gas_cost_f64 * best.1;
        let net_profit = spread - gas_cost_usdc;
        let roi = (net_profit / worst.1) * 100.0;

        println!("    💰  PROFIT ANALYSIS");
        println!("    ├─ Gross Profit:  {:.6}", spread);
        println!("    ├─ Net Profit:    {:.6}", net_profit);
        println!("    └─ ROI:           {:.2}%  📊", roi);

        let threshold = 2.0;
        println!("    VERDICT: {}",
            if net_profit > threshold {
                format!("✅ EXECUTE ${:.6}  (Threshold: ${})", net_profit, threshold).green()
            } else {
                format!("❌ SKIP ${:.6}  (Below Threshold)", net_profit).red()
            }
        );
    }

    Ok(())
}
