pub struct PriceBreakdown {
    pub base_price: f64,
    pub markup: f64,
    pub service_fee: f64,
    pub total: f64,
}

/// calculate the final price with markup and service fee
pub fn calculate_price(base: f64) -> PriceBreakdown {
    let markup_percentage = 0.05; // 5% markup
    let fixed_service_fee = 200.0; // fixed service fee

    let markup = base * markup_percentage;
    let total = base + markup + fixed_service_fee;

    PriceBreakdown {
        base_price: base,
        markup,
        service_fee: fixed_service_fee,
        total,
    }
}
