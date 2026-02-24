import { motion } from "framer-motion";
import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { createBooking } from "../services/api";
import { useTheme } from "../context/ThemeContext";
import { useAuth } from "../context/AuthContext";

export default function Booking() {
  const { id } = useParams();
  const navigate = useNavigate();
  const { theme } = useTheme();
  const { user } = useAuth();
  const [formData, setFormData] = useState({ name: "", email: "", paymentMethod: "usdt" });
  const [loading, setLoading] = useState(false);

  if (!user) {
    navigate("/login");
    return null;
  }

  const cryptoOptions = [
    { id: "usdt", name: "USDT", icon: "₮", chain: "Ethereum" },
    { id: "usdc", name: "USDC", icon: "$", chain: "Ethereum" },
    { id: "eth", name: "Ethereum", icon: "Ξ", chain: "Ethereum" },
    { id: "btc", name: "Bitcoin", icon: "₿", chain: "Bitcoin" },
    { id: "sol", name: "Solana", icon: "◎", chain: "Solana" },
    { id: "binance", name: "BNB", icon: "B", chain: "BSC" },
  ];

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    try {
      const res = await createBooking({ flight_id: id, ...formData });
      navigate(`/payment/${res.data.booking_id}`);
    } catch (err) {
      alert("Booking failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <motion.div
      className={`min-h-screen p-6 flex items-center justify-center ${theme === "light" ? "bg-gray-50" : ""}`}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
    >
      <motion.form
        onSubmit={handleSubmit}
        className={`glass-effect rounded-3xl p-8 max-w-2xl w-full border ${theme === "dark" ? "border-luxury-gold/30" : "border-gray-300 bg-white/50"}`}
        initial={{ scale: 0.9, y: 50 }}
        animate={{ scale: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        <h1 className={`text-4xl font-serif font-bold mb-8 text-center ${theme === "dark" ? "text-luxury-gold" : "text-gray-900"}`}>Complete Your Booking</h1>

        <div className="space-y-6">
          <motion.input
            type="text"
            placeholder="Full Name"
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            className={`w-full p-4 rounded-xl border focus:outline-none focus:ring-2 ${theme === "dark" ? "bg-luxury-charcoal/50 border-luxury-gold/20 text-luxury-ivory placeholder-luxury-platinum/50 focus:ring-luxury-gold" : "bg-white border-gray-300 text-gray-900 placeholder-gray-400 focus:ring-blue-500"}`}
            whileFocus={{ scale: 1.02 }}
            required
          />

          <motion.input
            type="email"
            placeholder="Email Address"
            value={formData.email}
            onChange={(e) => setFormData({ ...formData, email: e.target.value })}
            className={`w-full p-4 rounded-xl border focus:outline-none focus:ring-2 ${theme === "dark" ? "bg-luxury-charcoal/50 border-luxury-gold/20 text-luxury-ivory placeholder-luxury-platinum/50 focus:ring-luxury-gold" : "bg-white border-gray-300 text-gray-900 placeholder-gray-400 focus:ring-blue-500"}`}
            whileFocus={{ scale: 1.02 }}
            required
          />

          <div>
            <h3 className={`text-xl font-serif mb-4 ${theme === "dark" ? "text-luxury-gold" : "text-gray-900"}`}>Select Crypto Payment</h3>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
              {cryptoOptions.map((crypto) => (
                <motion.button
                  key={crypto.id}
                  type="button"
                  onClick={() => setFormData({ ...formData, paymentMethod: crypto.id })}
                  className={`p-4 rounded-xl border-2 transition-all ${
                    formData.paymentMethod === crypto.id
                      ? theme === "dark" ? "border-luxury-gold bg-luxury-gold/20" : "border-blue-600 bg-blue-50"
                      : theme === "dark" ? "border-luxury-gold/20 bg-luxury-charcoal/30" : "border-gray-300 bg-white"
                  }`}
                  whileHover={{ scale: 1.05 }}
                  whileTap={{ scale: 0.95 }}
                >
                  <div className="text-3xl mb-2">{crypto.icon}</div>
                  <div className={`font-bold ${theme === "dark" ? "text-luxury-ivory" : "text-gray-900"}`}>{crypto.name}</div>
                  <div className={`text-xs ${theme === "dark" ? "text-luxury-platinum" : "text-gray-600"}`}>{crypto.chain}</div>
                </motion.button>
              ))}
            </div>
          </div>

          <motion.button
            type="submit"
            disabled={loading}
            className={`w-full py-4 rounded-xl font-bold text-lg ${theme === "dark" ? "bg-gradient-to-r from-luxury-gold to-luxury-platinum text-luxury-midnight" : "bg-gradient-to-r from-blue-600 to-blue-800 text-white"}`}
            whileHover={{ scale: 1.02, boxShadow: "0 10px 40px rgba(212,175,55,0.4)" }}
            whileTap={{ scale: 0.98 }}
          >
            {loading ? "Processing..." : "Proceed to Payment"}
          </motion.button>
        </div>
      </motion.form>
    </motion.div>
  );
}
