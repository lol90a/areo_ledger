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
    /// Smallest unit decimals (for display/conversion only — never for arithmetic).
    pub fn decimals(self) -> u8 {
        match self {
            Self::Btc => 8,
            Self::Sol => 9,
            _ => 18,
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
        "binance" => Some((Chain::Bsc, CryptoToken::Bnb)),
        _ => None,
    }
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
    pub fn from_decimal_str(s: &str, currency: FiatCurrency) -> Result<Self, MoneyError> {
        let s = s.trim();
        let parts: Vec<&str> = s.splitn(2, '.').collect();
        let whole: i64 = parts[0]
            .parse()
            .map_err(|_| MoneyError::ParseError(s.to_string()))?;
        let frac: i64 = if parts.len() == 2 {
            let raw = format!("{:0<2}", &parts[1][..parts[1].len().min(2)]);
            raw.parse()
                .map_err(|_| MoneyError::ParseError(s.to_string()))?
        } else {
            0
        };
        Self::from_cents(whole * 100 + frac, currency)
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
