use std::fmt;

/// Supported fiat currencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FiatCurrency {
    Usd,
    Eur,
    Gbp,
}

impl FiatCurrency {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Usd => "USD",
            Self::Eur => "EUR",
            Self::Gbp => "GBP",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "USD" => Some(Self::Usd),
            "EUR" => Some(Self::Eur),
            "GBP" => Some(Self::Gbp),
            _ => None,
        }
    }
}

/// Supported crypto tokens — canonical enum, never raw strings inside the domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CryptoToken {
    Btc,
    Eth,
    Usdt,
    Usdc,
    Sol,
    Bnb,
}

impl CryptoToken {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Btc => "BTC",
            Self::Eth => "ETH",
            Self::Usdt => "USDT",
            Self::Usdc => "USDC",
            Self::Sol => "SOL",
            Self::Bnb => "BNB",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "BTC" => Some(Self::Btc),
            "ETH" => Some(Self::Eth),
            "USDT" => Some(Self::Usdt),
            "USDC" => Some(Self::Usdc),
            "SOL" => Some(Self::Sol),
            "BNB" => Some(Self::Bnb),
            _ => None,
        }
    }
    /// Number of decimal places in the token's smallest on-chain unit.
    /// BTC=8 (sats), SOL=9 (lamports), ETH/BNB=18 (wei), USDT/USDC=6 (ERC-20 stablecoins).
    pub fn decimals(self) -> u8 {
        match self {
            Self::Btc => 8,
            Self::Sol => 9,
            Self::Usdt | Self::Usdc => 6,
            Self::Eth | Self::Bnb => 18,
        }
    }

    /// True for ERC-20 tokens whose transfer value lives in contract logs, not the
    /// native `value` field of a transaction.
    pub fn is_erc20(self) -> bool {
        matches!(self, Self::Usdt | Self::Usdc)
    }

    /// True for fiat-pegged stablecoins (priced at ~1 USD).
    pub fn is_stablecoin(self) -> bool {
        matches!(self, Self::Usdt | Self::Usdc)
    }

    /// Canonical Ethereum-mainnet ERC-20 contract address for token-transfer
    /// verification. `None` for native assets. The `_contract` metadata is only
    /// used to locate the ERC-20 `Transfer` log for the configured receiver.
    pub fn erc20_contract(self) -> Option<&'static str> {
        match self {
            Self::Usdt => Some("0xdac17f958d2ee523a2206206994597c13d831ec7"),
            Self::Usdc => Some("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
            _ => None,
        }
    }

    /// CoinGecko id used by the live price oracle.
    pub fn coingecko_id(self) -> &'static str {
        match self {
            Self::Btc => "bitcoin",
            Self::Eth => "ethereum",
            Self::Usdt => "tether",
            Self::Usdc => "usd-coin",
            Self::Sol => "solana",
            Self::Bnb => "binancecoin",
        }
    }
}

/// Canonical blockchain network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Chain {
    Bitcoin,
    Ethereum,
    Solana,
    Bsc,
}

impl Chain {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bitcoin => "Bitcoin",
            Self::Ethereum => "Ethereum",
            Self::Solana => "Solana",
            Self::Bsc => "BSC",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Bitcoin" => Some(Self::Bitcoin),
            "Ethereum" => Some(Self::Ethereum),
            "Solana" => Some(Self::Solana),
            "BSC" => Some(Self::Bsc),
            _ => None,
        }
    }
}

/// Maps a payment method slug to its canonical (Chain, CryptoToken).
/// Single source of truth — replaces the scattered `match method` blocks.
pub fn resolve_chain_token(method: &str) -> Option<(Chain, CryptoToken)> {
    match method {
        "btc" => Some((Chain::Bitcoin, CryptoToken::Btc)),
        "eth" => Some((Chain::Ethereum, CryptoToken::Eth)),
        "usdt" => Some((Chain::Ethereum, CryptoToken::Usdt)),
        "usdc" => Some((Chain::Ethereum, CryptoToken::Usdc)),
        "sol" => Some((Chain::Solana, CryptoToken::Sol)),
        "bsc" | "bnb" | "binance" => Some((Chain::Bsc, CryptoToken::Bnb)),
        _ => None,
    }
}

// ── Crypto conversion helpers ─────────────────────────────────────────────────

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

/// Convert a fiat total into a crypto amount (whole token units) given the token's
/// current USD unit price. Rounds up to the token's smallest unit so the platform is
/// never underpaid by rounding. Returns `None` on a non-positive price.
pub fn usd_to_crypto_amount(
    usd: Decimal,
    unit_price_usd: Decimal,
    decimals: u8,
) -> Option<Decimal> {
    if unit_price_usd <= Decimal::ZERO {
        return None;
    }
    let raw = usd.checked_div(unit_price_usd)?;
    // Round the quote to the token's on-chain precision, rounding up (favor the platform).
    Some(raw.round_dp_with_strategy(
        decimals as u32,
        rust_decimal::RoundingStrategy::AwayFromZero,
    ))
}

/// Convert a whole-token decimal amount into the token's smallest integer unit
/// (wei / lamports / satoshis / ERC-20 base units).
pub fn crypto_base_units(amount: Decimal, decimals: u8) -> Option<u128> {
    if amount < Decimal::ZERO {
        return None;
    }
    let scale = Decimal::from(10u64.checked_pow(decimals as u32)?);
    let scaled = amount.checked_mul(scale)?;
    scaled.round().to_u128()
}

// ── Money ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyError {
    CurrencyMismatch {
        expected: FiatCurrency,
        got: FiatCurrency,
    },
    NegativeAmount,
    Overflow,
    ParseError(String),
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrencyMismatch { expected, got } => write!(
                f,
                "Currency mismatch: expected {} got {}",
                expected.as_str(),
                got.as_str()
            ),
            Self::NegativeAmount => write!(f, "Amount cannot be negative"),
            Self::Overflow => write!(f, "Arithmetic overflow"),
            Self::ParseError(s) => write!(f, "Parse error: {}", s),
        }
    }
}

/// Immutable monetary value stored as integer cents (smallest currency unit).
/// RULE: All financial arithmetic MUST go through this type. Never use f64 for money.
/// 1 USD = 100 cents. $9500.00 = 950_000 cents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Money {
    cents: i64,
    currency: FiatCurrency,
}

impl Money {
    /// Primary constructor — from integer cents.
    pub fn from_cents(cents: i64, currency: FiatCurrency) -> Result<Self, MoneyError> {
        if cents < 0 {
            return Err(MoneyError::NegativeAmount);
        }
        Ok(Self { cents, currency })
    }

    /// Construct from a decimal string "9500.00" — used ONLY at DB/API boundaries.
    /// Rejects negative and non-numeric input; never panics on multi-byte UTF-8.
    pub fn from_decimal_str(s: &str, currency: FiatCurrency) -> Result<Self, MoneyError> {
        let s = s.trim();
        if s.starts_with('-') {
            return Err(MoneyError::NegativeAmount);
        }
        let mut parts = s.splitn(2, '.');
        let whole_str = parts.next().unwrap_or("");
        let whole: i64 = whole_str
            .parse()
            .map_err(|_| MoneyError::ParseError(s.to_string()))?;

        let frac: i64 = match parts.next() {
            Some(frac_str) => {
                // Take at most the first two *characters* (cents), padding on the right.
                let mut digits = String::with_capacity(2);
                for ch in frac_str.chars().take(2) {
                    if !ch.is_ascii_digit() {
                        return Err(MoneyError::ParseError(s.to_string()));
                    }
                    digits.push(ch);
                }
                while digits.len() < 2 {
                    digits.push('0');
                }
                digits
                    .parse()
                    .map_err(|_| MoneyError::ParseError(s.to_string()))?
            }
            None => 0,
        };

        let cents = whole
            .checked_mul(100)
            .and_then(|v| v.checked_add(frac))
            .ok_or(MoneyError::Overflow)?;
        Self::from_cents(cents, currency)
    }

    pub fn zero(currency: FiatCurrency) -> Self {
        Self { cents: 0, currency }
    }

    pub fn cents(self) -> i64 {
        self.cents
    }
    pub fn currency(self) -> FiatCurrency {
        self.currency
    }
    pub fn is_zero(self) -> bool {
        self.cents == 0
    }

    /// Decimal string for serialization/display — never for arithmetic.
    pub fn to_decimal_str(self) -> String {
        format!("{}.{:02}", self.cents / 100, (self.cents % 100).abs())
    }

    /// Exact decimal value (currency units) for crypto conversion math.
    pub fn to_decimal(self) -> rust_decimal::Decimal {
        rust_decimal::Decimal::from(self.cents) / rust_decimal::Decimal::from(100)
    }

    pub fn checked_add(self, other: Self) -> Result<Self, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency,
                got: other.currency,
            });
        }
        let sum = self
            .cents
            .checked_add(other.cents)
            .ok_or(MoneyError::Overflow)?;
        Self::from_cents(sum, self.currency)
    }

    pub fn checked_sub(self, other: Self) -> Result<Self, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency,
                got: other.currency,
            });
        }
        let diff = self
            .cents
            .checked_sub(other.cents)
            .ok_or(MoneyError::NegativeAmount)?;
        Self::from_cents(diff, self.currency)
    }

    /// Apply a basis-point rate. 1000 bps = 10.00%, 500 bps = 5.00%.
    /// This is the ONLY way to compute percentages on Money.
    pub fn apply_bps(self, basis_points: u32) -> Result<Self, MoneyError> {
        let result = self
            .cents
            .checked_mul(basis_points as i64)
            .and_then(|v| v.checked_div(10_000))
            .ok_or(MoneyError::Overflow)?;
        Self::from_cents(result, self.currency)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.to_decimal_str(), self.currency.as_str())
    }
}

// ── PriceBreakdown (rebuilt on Money) ─────────────────────────────────────────

/// Immutable, self-validating price breakdown.
/// Markup = 1000 bps (10%), service fee = 500 bps (5%).
#[derive(Debug, Clone, Copy)]
pub struct PriceBreakdown {
    pub base: Money,
    pub markup: Money,
    pub service_fee: Money,
    pub total: Money,
}

impl PriceBreakdown {
    pub const MARKUP_BPS: u32 = 1_000;
    pub const SERVICE_FEE_BPS: u32 = 500;

    pub fn calculate(base: Money) -> Result<Self, MoneyError> {
        let markup = base.apply_bps(Self::MARKUP_BPS)?;
        let service_fee = base.apply_bps(Self::SERVICE_FEE_BPS)?;
        let total = base.checked_add(markup)?.checked_add(service_fee)?;
        Ok(Self {
            base,
            markup,
            service_fee,
            total,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakdown_is_exact_integer_arithmetic() {
        let base = Money::from_cents(1_000_00, FiatCurrency::Usd).unwrap(); // $1000.00
        let bd = PriceBreakdown::calculate(base).unwrap();
        assert_eq!(bd.markup.cents(), 100_00);
        assert_eq!(bd.service_fee.cents(), 50_00);
        assert_eq!(bd.total.cents(), 1_150_00);
    }

    #[test]
    fn currency_mismatch_rejected() {
        let a = Money::from_cents(100, FiatCurrency::Usd).unwrap();
        let b = Money::from_cents(100, FiatCurrency::Eur).unwrap();
        assert!(matches!(
            a.checked_add(b),
            Err(MoneyError::CurrencyMismatch { .. })
        ));
    }

    #[test]
    fn negative_amount_rejected() {
        assert!(Money::from_cents(-1, FiatCurrency::Usd).is_err());
    }
}
