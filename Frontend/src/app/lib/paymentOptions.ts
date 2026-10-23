import { fetchPaymentOptions as fetchPaymentOptionsFromBackend, type PaymentOption } from "./backend";

export type { PaymentOption };

/**
 * Shown only when the backend is unreachable, and every route is marked
 * `available: false`.
 *
 * These entries exist so the network picker renders rather than collapsing into an
 * empty page. They deliberately carry no wallet address and no chain id: a deposit
 * address or chain id invented in the browser is exactly how funds end up on the
 * wrong network, so checkout refuses to quote against a fallback route.
 */
export const FALLBACK_PAYMENT_OPTIONS: PaymentOption[] = [
  {
    method: "btc",
    token: "BTC",
    chain: "Bitcoin",
    display_name: "Bitcoin (BTC)",
    wallet_address: null,
    available: false,
    wallet_kind: "manual",
    chain_id: null,
    token_contract: null,
    decimals: 8,
  },
  {
    method: "eth",
    token: "ETH",
    chain: "Ethereum",
    display_name: "Ethereum (ETH)",
    wallet_address: null,
    available: false,
    wallet_kind: "evm",
    chain_id: null,
    token_contract: null,
    decimals: 18,
  },
  {
    method: "usdt",
    token: "USDT",
    chain: "Ethereum",
    display_name: "Tether USD (USDT)",
    wallet_address: null,
    available: false,
    wallet_kind: "evm",
    chain_id: null,
    token_contract: null,
    decimals: 6,
  },
  {
    method: "usdc",
    token: "USDC",
    chain: "Ethereum",
    display_name: "USD Coin (USDC)",
    wallet_address: null,
    available: false,
    wallet_kind: "evm",
    chain_id: null,
    token_contract: null,
    decimals: 6,
  },
  {
    method: "bsc",
    token: "BNB",
    chain: "BSC",
    display_name: "BNB on BSC",
    wallet_address: null,
    available: false,
    wallet_kind: "evm",
    chain_id: null,
    token_contract: null,
    decimals: 18,
  },
  {
    method: "sol",
    token: "SOL",
    chain: "Solana",
    display_name: "Solana (SOL)",
    wallet_address: null,
    available: false,
    wallet_kind: "solana",
    chain_id: null,
    token_contract: null,
    decimals: 9,
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
