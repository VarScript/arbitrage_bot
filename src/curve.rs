use crate::tokens::TokenPair;
use anyhow::{anyhow, Context, Result};
use ethers::{abi::Abi, prelude::*};
use once_cell::sync::Lazy;
use std::sync::Arc;

// Token addresses in Curve pools
const CURVE_3POOL: &str = "0xbEbc44782C7dB0a1A60Cb6fe97d0b483032FF1C7";
const CURVE_SUSD: &str = "0xA5407eAE9Ba41422680e2e00537571bcC53efBfD";
const CURVE_TRICRYPTO: &str = "0xD51a44d3FaE010294C616388b506AcdA1bfAAE46";
const CURVE_STETH: &str = "0xDC24316b9AE028F1497c275EB9192a3Ea0f22A61";

const DAI: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
const USDT: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
const SUSD: &str = "0x57Ab1ec28D129707052df4dF418D58a2D46d5f51";
const WBTC: &str = "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599";
const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
const STETH: &str = "0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84";

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub name: &'static str,
    pub address: Address,
    pub token_addresses: [Address; 3],
    pub indices: [i128; 3],
    pub is_tricrypto: bool,
}

static POOL_3: Lazy<PoolConfig> = Lazy::new(|| PoolConfig {
    name: "3pool",
    address: CURVE_3POOL.parse().unwrap(),
    token_addresses: [DAI.parse().unwrap(), USDC.parse().unwrap(), USDT.parse().unwrap()],
    indices: [0, 1, 2],
    is_tricrypto: false,
});

static POOL_SUSD: Lazy<PoolConfig> = Lazy::new(|| PoolConfig {
    name: "sUSD",
    address: CURVE_SUSD.parse().unwrap(),
    token_addresses: [SUSD.parse().unwrap(), USDC.parse().unwrap(), USDT.parse().unwrap()],
    indices: [0, 1, 2],
    is_tricrypto: false,
});

static POOL_TRICRYPTO: Lazy<PoolConfig> = Lazy::new(|| PoolConfig {
    name: "TriCrypto",
    address: CURVE_TRICRYPTO.parse().unwrap(),
    token_addresses: [WBTC.parse().unwrap(), WETH.parse().unwrap(), USDT.parse().unwrap()],
    indices: [0, 1, 2],
    is_tricrypto: true,
});

static POOL_STETH: Lazy<PoolConfig> = Lazy::new(|| PoolConfig {
    name: "stETH",
    address: CURVE_STETH.parse().unwrap(),
    token_addresses: [STETH.parse().unwrap(), WETH.parse().unwrap(), WETH.parse().unwrap()],
    indices: [0, 1, 1],
    is_tricrypto: false,
});

static POOLS: Lazy<Vec<&'static PoolConfig>> = Lazy::new(|| vec![&POOL_3, &POOL_SUSD, &POOL_TRICRYPTO, &POOL_STETH]);

pub struct CurvePrice {
    pub pool_name: String,
    pub price: f64,
    pub token_pair: String,
}

pub async fn get_price_from_curve(
    client: Arc<Provider<Http>>,
    pool_config: &PoolConfig,
    token_pair: &TokenPair,
) -> Result<f64> {
    if pool_config.name == "TriCrypto" && token_pair.name == "ETH/USDT" {
        return Err(anyhow!("Skipping ETH/USDT on TriCrypto due to revert issues"));
    }

    let abi_str = if pool_config.is_tricrypto {
        include_str!("../abis/curve_tricrypto_abi.json")
    } else {
        include_str!("../abis/curve_abi.json")
    };
    let abi_curve: Abi = serde_json::from_str(abi_str)?;
    let contract = Contract::new(pool_config.address, abi_curve, client);

    let base_index = pool_config
        .token_addresses
        .iter()
        .position(|&t| t == token_pair.base_token)
        .map(|i| pool_config.indices[i])
        .context("Base token not found in pool")?;

    let quote_index = pool_config
        .token_addresses
        .iter()
        .position(|&t| t == token_pair.quote_token)
        .map(|i| pool_config.indices[i])
        .context("Quote token not found in pool")?;

    let amount_in = U256::exp10(token_pair.base_decimals.into());
    let method_name = "get_dy";

    if pool_config.is_tricrypto {
        let method = contract.method::<(U256, U256, U256), U256>(
            method_name,
            (base_index.into(), quote_index.into(), amount_in),
        )?;
        let amount_out = method.call().await?;
        let quote_amount = amount_out.as_u128() as f64
            / 10f64.powi(token_pair.quote_decimals as i32);
        Ok(quote_amount)
    } else {
        let method = contract.method::<(i128, i128, U256), U256>(
            method_name,
            (base_index, quote_index, amount_in),
        )?;
        let amount_out = method.call().await?;
        let quote_amount = amount_out.as_u128() as f64
            / 10f64.powi(token_pair.quote_decimals as i32);
        Ok(quote_amount)
    }
}

pub async fn get_prices_for_all_pairs(
    client: Arc<Provider<Http>>,
    token_pairs: &[TokenPair],
) -> Vec<CurvePrice> {
    let mut prices = Vec::new();

    for &pool_config in POOLS.iter() {
        let relevant_pairs: Vec<&TokenPair> = token_pairs
            .iter()
            .filter(|pair| {
                pool_config.token_addresses.contains(&pair.base_token)
                    && pool_config.token_addresses.contains(&pair.quote_token)
            })
            .collect();

        for pair in relevant_pairs {
            if let Ok(price) = get_price_from_curve(client.clone(), pool_config, pair).await {
                prices.push(CurvePrice {
                    pool_name: pool_config.name.to_string(),
                    price,
                    token_pair: pair.name.clone(),
                });
            }
        }
    }

    prices
}