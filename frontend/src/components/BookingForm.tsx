import { useState } from "react";
import { createBooking } from "../services/api";

export default function BookingForm() {
  const [flightId, setFlightId] = useState("");
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const res = await createBooking({
        flight_id: flightId,
        name,
        email,
      });

      alert(`Booking created with ID: ${res.data.booking_id}`);
    } catch (err: any) {
      setError("Failed to create booking. Please try again.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <form
      onSubmit={handleSubmit}
      className="bg-white shadow-lg rounded-xl p-6 flex flex-col gap-4"
    >
      <h2 className="text-xl font-bold text-gray-800">Booking Form</h2>

      {error && (
        <div className="bg-red-100 text-red-700 p-2 rounded">{error}</div>
      )}

      <input
        type="text"
        placeholder="Flight ID"
        value={flightId}
        onChange={(e) => setFlightId(e.target.value)}
        className="border p-2 rounded focus:outline-none focus:ring focus:ring-blue-300"
        required
      />

      <input
        type="text"
        placeholder="Full Name"
        value={name}
        onChange={(e) => setName(e.target.value)}
        className="border p-2 rounded focus:outline-none focus:ring focus:ring-blue-300"
        required
      />

      <input
        type="email"
        placeholder="Email Address"
        value={email}
        onChange={(e) => setEmail(e.target.value)}
        className="border p-2 rounded focus:outline-none focus:ring focus:ring-blue-300"
        required
      />

      <button
        type="submit"
        disabled={loading}
        className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition disabled:opacity-50"
      >
        {loading ? "Processing..." : "Create Booking"}
      </button>
    </form>
  );
}
