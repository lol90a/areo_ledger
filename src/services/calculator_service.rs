use std::collections::HashMap;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct CurrencyInfo {
    pub name: String,
    pub price: f64,
    pub symbol: String,
}

pub struct CalculatorService;

impl CalculatorService {
    pub fn convert(amount: f64, from_currency: &str, to_currency: &str) -> (f64, f64) {
        let crypto_prices = Self::get_crypto_prices();
        let fiat_rates = Self::get_fiat_rates();
        
        let from_price = crypto_prices.get(from_currency)
            .map(|c| c.price)
            .or_else(|| fiat_rates.get(from_currency).map(|f| f.price))
            .unwrap_or(1.0);
        
        let to_price = crypto_prices.get(to_currency)
            .map(|c| c.price)
            .or_else(|| fiat_rates.get(to_currency).map(|f| f.price))
            .unwrap_or(1.0);
        
        let result = (amount * from_price) / to_price;
        let rate = from_price / to_price;

        (result, rate)
    }

    pub fn get_crypto_prices() -> HashMap<String, CurrencyInfo> {
        let mut prices = HashMap::new();
        prices.insert("BTC".to_string(), CurrencyInfo { name: "Bitcoin".to_string(), price: 45000.0, symbol: "₿".to_string() });
        prices.insert("ETH".to_string(), CurrencyInfo { name: "Ethereum".to_string(), price: 2500.0, symbol: "Ξ".to_string() });
        prices.insert("USDT".to_string(), CurrencyInfo { name: "Tether".to_string(), price: 1.0, symbol: "₮".to_string() });
        prices.insert("USDC".to_string(), CurrencyInfo { name: "USD Coin".to_string(), price: 1.0, symbol: "$".to_string() });
        prices.insert("SOL".to_string(), CurrencyInfo { name: "Solana".to_string(), price: 100.0, symbol: "◎".to_string() });
        prices.insert("BNB".to_string(), CurrencyInfo { name: "Binance Coin".to_string(), price: 320.0, symbol: "BNB".to_string() });
        prices.insert("ADA".to_string(), CurrencyInfo { name: "Cardano".to_string(), price: 0.45, symbol: "ADA".to_string() });
        prices.insert("DOGE".to_string(), CurrencyInfo { name: "Dogecoin".to_string(), price: 0.08, symbol: "DOGE".to_string() });
        prices.insert("XRP".to_string(), CurrencyInfo { name: "Ripple".to_string(), price: 0.60, symbol: "XRP".to_string() });
        prices.insert("DOT".to_string(), CurrencyInfo { name: "Polkadot".to_string(), price: 7.5, symbol: "DOT".to_string() });
        prices.insert("MATIC".to_string(), CurrencyInfo { name: "Polygon".to_string(), price: 0.85, symbol: "MATIC".to_string() });
        prices.insert("LINK".to_string(), CurrencyInfo { name: "Chainlink".to_string(), price: 15.0, symbol: "LINK".to_string() });
        prices.insert("LTC".to_string(), CurrencyInfo { name: "Litecoin".to_string(), price: 72.0, symbol: "Ł".to_string() });
        prices.insert("AVAX".to_string(), CurrencyInfo { name: "Avalanche".to_string(), price: 38.0, symbol: "AVAX".to_string() });
        prices.insert("UNI".to_string(), CurrencyInfo { name: "Uniswap".to_string(), price: 6.5, symbol: "UNI".to_string() });
        prices
    }

    pub fn get_fiat_rates() -> HashMap<String, CurrencyInfo> {
        let mut rates = HashMap::new();
        rates.insert("USD".to_string(), CurrencyInfo { name: "US Dollar".to_string(), price: 1.0, symbol: "$".to_string() });
        rates.insert("EUR".to_string(), CurrencyInfo { name: "Euro".to_string(), price: 0.92, symbol: "€".to_string() });
        rates.insert("GBP".to_string(), CurrencyInfo { name: "British Pound".to_string(), price: 0.79, symbol: "£".to_string() });
        rates.insert("JPY".to_string(), CurrencyInfo { name: "Japanese Yen".to_string(), price: 149.50, symbol: "¥".to_string() });
        rates.insert("CNY".to_string(), CurrencyInfo { name: "Chinese Yuan".to_string(), price: 7.24, symbol: "¥".to_string() });
        rates.insert("AUD".to_string(), CurrencyInfo { name: "Australian Dollar".to_string(), price: 1.53, symbol: "A$".to_string() });
        rates.insert("CAD".to_string(), CurrencyInfo { name: "Canadian Dollar".to_string(), price: 1.36, symbol: "C$".to_string() });
        rates.insert("CHF".to_string(), CurrencyInfo { name: "Swiss Franc".to_string(), price: 0.88, symbol: "CHF".to_string() });
        rates.insert("INR".to_string(), CurrencyInfo { name: "Indian Rupee".to_string(), price: 83.12, symbol: "₹".to_string() });
        rates.insert("KRW".to_string(), CurrencyInfo { name: "South Korean Won".to_string(), price: 1320.0, symbol: "₩".to_string() });
        rates.insert("MXN".to_string(), CurrencyInfo { name: "Mexican Peso".to_string(), price: 17.15, symbol: "MX$".to_string() });
        rates.insert("BRL".to_string(), CurrencyInfo { name: "Brazilian Real".to_string(), price: 4.98, symbol: "R$".to_string() });
        rates.insert("RUB".to_string(), CurrencyInfo { name: "Russian Ruble".to_string(), price: 92.50, symbol: "₽".to_string() });
        rates.insert("SGD".to_string(), CurrencyInfo { name: "Singapore Dollar".to_string(), price: 1.34, symbol: "S$".to_string() });
        rates.insert("HKD".to_string(), CurrencyInfo { name: "Hong Kong Dollar".to_string(), price: 7.83, symbol: "HK$".to_string() });
        rates
    }
}
