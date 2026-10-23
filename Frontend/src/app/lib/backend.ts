import { apiRequest } from "./api";
import type { AuthSession } from "./auth";

export type PaymentOption = {
  method: string;
  token: string;
  chain: string;
  display_name: string;
  wallet_address: string | null;
  available: boolean;
  /** Which browser wallet family handles this route. */
  wallet_kind: "evm" | "solana" | "manual";
  /** EIP-155 chain id for EVM routes. Server-supplied, never hard-coded here. */
  chain_id: number | null;
  /** ERC-20 contract for non-native EVM tokens. Server-supplied. */
  token_contract: string | null;
  decimals: number;
};

export type CreateBookingResponse = {
  booking_id: string;
  /** Integer minor units. Use this, never the float, for any comparison. */
  total_cents: number;
  currency: string;
  total_price: number;
};

/**
 * A server-issued, expiring price quote.
 *
 * `total_cents` and `crypto_base_units` are the figures the user approves, and the
 * exact values that must be echoed back on acceptance. The browser never computes
 * either of them.
 */
export type QuoteResponse = {
  quote_id: string;
  booking_id: string;
  status: "active" | "accepted" | "superseded" | "expired";
  method: string;
  chain: string;
  token: string;
  token_decimals: number;
  base_cents: number;
  markup_cents: number;
  service_fee_cents: number;
  total_cents: number;
  currency: string;
  total_display: string;
  unit_price_usd: string;
  rate_source: string;
  rate_observed_at: string;
  crypto_amount: string;
  crypto_base_units: string;
  receiver_address: string;
  expires_at: string;
  seconds_remaining: number;
  /** The catalogue price moved since this booking was created. */
  price_changed: boolean;
  previous_total_cents: number | null;
};

export type LatestBookingResponse = {
  booking_id: string;
  flight_id: string | null;
  asset_id: string | null;
  status: string;
  base_price: number;
  total_price: number;
  base_cents: number;
  total_cents: number;
  currency: string;
  payment_method: string;
  created_at: string;
};

export type TransactionResponse = {
  id: string;
  type: string;
  asset: string;
  amount: number;
  status: string;
  date: string;
  tx_hash: string;
};

export type InitPaymentResponse = {
  wallet_address: string;
  token: string;
  chain: string;
  crypto_amount: string;
  /** The authoritative settlement figure, in the token's smallest unit. */
  crypto_base_units: string;
  total_cents: number;
  currency: string;
  quote_id: string;
};

export type FlightCatalogItem = {
  id: string;
  route: string;
  origin: string;
  destination: string;
  aircraft: string;
  price_usd: number;
  duration: string | null;
  seats: number;
  status: string;
};

export type AssetCatalogItem = {
  id: string;
  name: string;
  type: string;
  description: string | null;
  image: string | null;
  total_value: number;
  token_price: number;
  available_tokens: number;
  total_tokens: number;
  status: string;
};

export type PaymentProofUploadResponse = {
  message: string;
  booking_id: string;
  proof_path: string;
};

/**
 * Every state the checkout can truthfully be in.
 *
 * There is deliberately no state that a timer can advance into: each one is
 * reached by a real event (a server response, a wallet result, a confirmed
 * transaction). `payment_verified` only ever comes from the backend having
 * checked the chain.
 */
export type CheckoutPhase =
  | "idle"
  | "quote_requested"
  | "quote_ready"
  | "quote_expired"
  | "price_changed"
  | "quote_accepted"
  | "awaiting_wallet"
  | "insufficient_balance"
  | "wrong_network"
  | "transaction_submitted"
  | "pending_confirmation"
  | "payment_verified"
  | "failed";

export const CHECKOUT_PHASE_LABELS: Record<CheckoutPhase, string> = {
  idle: "Not started",
  quote_requested: "Requesting price",
  quote_ready: "Quote ready - review and approve",
  quote_expired: "Quote expired - request a new price",
  price_changed: "Price changed - approve the new amount",
  quote_accepted: "Price locked",
  awaiting_wallet: "Waiting for your wallet",
  insufficient_balance: "Insufficient wallet balance",
  wrong_network: "Wallet is on the wrong network",
  transaction_submitted: "Transaction submitted",
  pending_confirmation: "Waiting for blockchain confirmation",
  payment_verified: "Payment verified on-chain",
  failed: "Payment failed",
};

/**
 * UI-facing settlement status. The backend vocabulary is
 * `pending | confirmed | cancelled` for bookings and
 * `pending | confirmed | failed` for payments/transactions.
 * The UI renders four visual states, so `pending` maps to
 * `awaiting` and both `cancelled`/`failed` map to `failed`.
 */
export type SettlementStatus = "initialized" | "awaiting" | "confirmed" | "failed";

export function normalizeSettlementStatus(status: string | null | undefined): SettlementStatus {
  switch ((status ?? "").toLowerCase()) {
    case "confirmed":
      return "confirmed";
    case "cancelled":
    case "canceled":
    case "failed":
      return "failed";
    case "pending":
      return "awaiting";
    // Legacy / initialization states surface as the initial step.
    case "initialized":
      return "initialized";
    default:
      return "awaiting";
  }
}

export async function signup(payload: { email: string; name: string; password: string }) {
  return apiRequest<AuthSession>("/api/users/signup", {
    method: "POST",
    body: payload,
  });
}

export async function login(payload: { email: string; password: string }) {
  return apiRequest<AuthSession>("/api/users/login", {
    method: "POST",
    body: payload,
  });
}

export async function fetchPaymentOptions() {
  const data = await apiRequest<{ options: PaymentOption[] }>("/api/payments/options");
  return data.options;
}

export async function fetchFlights() {
  const data = await apiRequest<{ flights: FlightCatalogItem[] }>("/api/flights");
  return data.flights;
}

export async function fetchAssets() {
  const data = await apiRequest<{ assets: AssetCatalogItem[] }>("/api/assets");
  return data.assets;
}

export async function createBooking(
  token: string,
  payload: { user_id: string; flight_id?: string; asset_id?: string; payment_method: string },
) {
  return apiRequest<CreateBookingResponse>("/api/bookings", {
    method: "POST",
    token,
    body: payload,
  });
}

export async function fetchLatestBooking(token: string) {
  return apiRequest<LatestBookingResponse>("/api/bookings/latest", {
    method: "GET",
    token,
  });
}

export async function fetchBookingById(token: string, id: string) {
  return apiRequest<LatestBookingResponse>(`/api/bookings/${id}`, {
    method: "GET",
    token,
  });
}

/**
 * Ask the server for a fresh price. This is where a catalogue price change is
 * detected: the response carries `price_changed` and the previous total.
 */
export async function createQuote(token: string, payload: { booking_id: string; method: string }) {
  return apiRequest<QuoteResponse>("/api/payments/quote", {
    method: "POST",
    token,
    body: payload,
  });
}

/**
 * Approve a quote by echoing back the exact figures that were displayed.
 *
 * If either has moved, the server refuses with 409 rather than charging an amount
 * the user never saw. The caller must then show the new quote and ask again.
 */
export async function acceptQuote(
  token: string,
  payload: {
    booking_id: string;
    quote_id: string;
    approved_total_cents: number;
    approved_crypto_base_units: string;
  },
) {
  return apiRequest<QuoteResponse>("/api/payments/quote/accept", {
    method: "POST",
    token,
    body: payload,
  });
}

/** Turn the accepted quote into a payment record. Carries no amount: the server
 *  copies it from the quote the user approved. */
export async function initPayment(token: string, payload: { booking_id: string }) {
  return apiRequest<InitPaymentResponse>("/api/payments/init", {
    method: "POST",
    token,
    body: payload,
  });
}

export async function confirmPayment(token: string, payload: { booking_id: string; tx_hash: string }) {
  return apiRequest<{ message: string }>("/api/payments/confirm", {
    method: "POST",
    token,
    body: payload,
  });
}

export async function uploadPaymentProof(token: string, bookingId: string, file: File) {
  const formData = new FormData();
  formData.append("proof", file);

  return apiRequest<PaymentProofUploadResponse>(`/api/payments/proof/${bookingId}`, {
    method: "POST",
    token,
    body: formData,
  });
}

export async function fetchTransactions(token: string, userId: string) {
  return apiRequest<TransactionResponse[]>(`/api/transactions/${userId}`, {
    method: "GET",
    token,
  });
}
