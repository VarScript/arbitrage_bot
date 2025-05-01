use ethers::types::Address;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct TokenPair {
    pub base_token: Address,
    pub quote_token: Address,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub name: String,
}

impl TokenPair {
    pub fn new(
        base_token: &str,
        quote_token: &str,
        base_decimals: u8,
        quote_decimals: u8,
        name: &str,
    ) -> Self {
        Self {
            base_token: Address::from_str(base_token).unwrap(),
            quote_token: Address::from_str(quote_token).unwrap(),
            base_decimals,
            quote_decimals,
            name: name.to_string(),
        }
    }
}

pub struct TokenPairs {
    pub pairs: Vec<TokenPair>,
}

impl TokenPairs {
    pub fn new() -> Self {
        Self {
            pairs: vec![
                // ETH/USDC
                TokenPair::new(
                    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
                    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
                    18,
                    6,
                    "ETH/USDC",
                ),
                // ETH/USDT
                TokenPair::new(
                    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
                    "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT
                    18,
                    6,
                    "ETH/USDT",
                ),
                // WBTC/ETH
                TokenPair::new(
                    "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599", // WBTC
                    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
                    8,
                    18,
                    "WBTC/ETH",
                ),
                // LINK/ETH
                TokenPair::new(
                    "0x514910771AF9Ca656af840dff83E8264EcF986CA", // LINK
                    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
                    18,
                    18,
                    "LINK/ETH",
                ),
                // UNI/ETH
                TokenPair::new(
                    "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984", // UNI
                    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
                    18,
                    18,
                    "UNI/ETH",
                ),
            ],
        }
    }
}