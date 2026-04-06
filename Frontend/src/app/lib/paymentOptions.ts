import { fetchPaymentOptions as fetchPaymentOptionsFromBackend, type PaymentOption } from "./backend";

export type { PaymentOption };

export const FALLBACK_PAYMENT_OPTIONS: PaymentOption[] = [
  {
    method: "btc",
    token: "BTC",
    chain: "Bitcoin",
    display_name: "Bitcoin (BTC)",
    wallet_address: null,
    available: true,
  },
  {
    method: "eth",
    token: "ETH",
    chain: "Ethereum",
    display_name: "Ethereum (ETH)",
    wallet_address: null,
    available: true,
  },
  {
    method: "usdt",
    token: "USDT",
    chain: "Ethereum",
    display_name: "Tether USD (USDT)",
    wallet_address: null,
    available: true,
  },
  {
    method: "usdc",
    token: "USDC",
    chain: "Ethereum",
    display_name: "USD Coin (USDC)",
    wallet_address: null,
    available: true,
  },
  {
    method: "binance",
    token: "BNB",
    chain: "BSC",
    display_name: "BNB on BSC",
    wallet_address: null,
    available: true,
  },
  {
    method: "sol",
    token: "SOL",
    chain: "Solana",
    display_name: "Solana (SOL)",
    wallet_address: null,
    available: true,
  },
];

export async function fetchPaymentOptions(): Promise<PaymentOption[]> {
  try {
    const options = await fetchPaymentOptionsFromBackend();
    if (Array.isArray(options) && options.length > 0) {
      return options;
    }
  } catch (_error) {
    return FALLBACK_PAYMENT_OPTIONS;
  }

  return FALLBACK_PAYMENT_OPTIONS;
}

export function paymentOptionSummary(options: PaymentOption[]): string {
  return options
    .map((option) => (option.chain === "BSC" ? `${option.token}/${option.chain}` : option.token))
    .join(", ");
}
