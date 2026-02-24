import { useState } from "react";
import { initPayment, confirmPayment } from "../services/api";

interface Props {
  bookingId: string;
}

const METHODS = [
  { label: "USDT", value: "USDT" },
  { label: "USDC", value: "USDC" },
  { label: "Ethereum (ETH)", value: "ETH" },
  { label: "Solana (SOL)", value: "SOL" },
  { label: "Bitcoin (BTC)", value: "BTC" },
  { label: "Binance Pay", value: "BINANCE" },
];

export default function PaymentForm({ bookingId }: Props) {
  const [method, setMethod] = useState("USDT");
  const [status, setStatus] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handlePayment = async () => {
    setLoading(true);
    setStatus(null);

    try {
      const initRes = await initPayment({
        booking_id: bookingId,
        method,
      });

      const paymentId = initRes.data.payment_id;

      await confirmPayment(paymentId);

      setStatus("✅ Payment confirmed successfully!");
    } catch (err) {
      setStatus("❌ Payment failed. Please try again.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="bg-white shadow-lg rounded-xl p-6 flex flex-col gap-4">
      <h2 className="text-xl font-bold text-gray-800">Choose Payment Method</h2>

      <select
        value={method}
        onChange={(e) => setMethod(e.target.value)}
        className="border p-2 rounded focus:outline-none focus:ring focus:ring-blue-300"
      >
        {METHODS.map((m) => (
          <option key={m.value} value={m.value}>
            {m.label}
          </option>
        ))}
      </select>

      <button
        onClick={handlePayment}
        disabled={loading}
        className="bg-green-600 text-white px-4 py-2 rounded-lg hover:bg-green-700 transition disabled:opacity-50"
      >
        {loading ? "Processing Payment..." : "Pay Now"}
      </button>

      {status && <div className="text-center font-semibold mt-2">{status}</div>}
    </div>
  );
}
