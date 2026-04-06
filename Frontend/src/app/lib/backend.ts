import { apiRequest } from "./api";
import type { AuthSession } from "./auth";

export type PaymentOption = {
  method: string;
  token: string;
  chain: string;
  display_name: string;
  wallet_address: string | null;
  available: boolean;
};

export type CreateBookingResponse = {
  booking_id: string;
  total_price: number;
};

export type LatestBookingResponse = {
  booking_id: string;
  flight_id: string | null;
  status: string;
  base_price: number;
  total_price: number;
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
  amount: number;
};

export type PaymentProofUploadResponse = {
  message: string;
  booking_id: string;
  proof_path: string;
};

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

export async function createBooking(
  token: string,
  payload: { user_id: string; flight_id?: string; base_price: number; payment_method: string },
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

export async function initPayment(token: string, payload: { booking_id: string; method: string }) {
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
