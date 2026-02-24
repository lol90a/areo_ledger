import { motion } from "framer-motion";
import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { initPayment, confirmPayment } from "../services/api";
import { useTheme } from "../context/ThemeContext";

export default function Payment() {
  const { bookingId } = useParams();
  const navigate = useNavigate();
  const [step, setStep] = useState(1);
  const [paymentData, setPaymentData] = useState<any>(null);
  const [formData, setFormData] = useState({ walletAddress: "", amount: "", txHash: "" });
  const [loading, setLoading] = useState(false);
  const { theme } = useTheme();

  useEffect(() => {
    if (bookingId) {
      initPayment({ booking_id: bookingId, method: "usdt" })
        .then((res) => {
          setPaymentData(res.data);
          setFormData({ ...formData, amount: res.data.amount });
        })
        .catch(() => alert("Failed to initialize payment"));
    }
  }, [bookingId]);

  const handleConfirm = async () => {
    if (!formData.walletAddress || !formData.txHash) {
      alert("Please fill all fields");
      return;
    }
    setLoading(true);
    try {
      await confirmPayment({ booking_id: bookingId!, tx_hash: formData.txHash });
      setStep(3);
    } catch (err) {
      alert("Payment confirmation failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <motion.div
      className="min-h-screen p-6 flex items-center justify-center bg-gradient-to-br from-purple-900 via-indigo-900 to-black"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
    >
      <motion.div
        className="glass-effect rounded-3xl p-8 max-w-2xl w-full border-2 border-purple-500/50 relative z-10 bg-black/40"
        initial={{ scale: 0.9, rotateX: 20 }}
        animate={{ scale: 1, rotateX: 0 }}
        transition={{ duration: 0.6 }}
      >
        {step === 1 && paymentData && (
          <>
            <motion.div className="text-center mb-8">
              <motion.div
                className="text-7xl mb-4"
                animate={{ rotate: [0, 360] }}
                transition={{ duration: 3, repeat: Infinity, ease: "linear" }}
              >
                💎
              </motion.div>
              <h1 className="text-5xl font-serif font-bold text-transparent bg-clip-text bg-gradient-to-r from-purple-400 via-pink-400 to-purple-400 mb-2">Crypto Payment</h1>
              <p className="text-purple-300">Secure blockchain transaction</p>
            </motion.div>
            
            <div className="bg-gradient-to-br from-purple-900/50 to-indigo-900/50 rounded-2xl p-6 mb-6 border border-purple-500/30">
              <h3 className="text-xl font-bold text-purple-300 mb-4">Payment Details</h3>
              <div className="space-y-3 text-purple-100">
                <div className="flex justify-between items-center">
                  <span>Token:</span>
                  <span className="font-bold text-2xl text-transparent bg-clip-text bg-gradient-to-r from-yellow-400 to-pink-400">{paymentData.token}</span>
                </div>
                <div className="flex justify-between items-center">
                  <span>Network:</span>
                  <span className="font-bold text-purple-400">{paymentData.chain}</span>
                </div>
                <div className="flex justify-between items-center">
                  <span>Amount:</span>
                  <span className="font-bold text-3xl text-transparent bg-clip-text bg-gradient-to-r from-green-400 to-emerald-400">{paymentData.amount} {paymentData.token}</span>
                </div>
              </div>
            </div>

            <div className="bg-gradient-to-br from-indigo-900/50 to-purple-900/50 rounded-2xl p-6 mb-6 border border-purple-500/30">
              <h3 className="text-lg font-bold text-purple-300 mb-3">Recipient Address</h3>
              <div className="bg-black/70 p-4 rounded-xl break-all font-mono text-sm text-purple-200 border border-purple-500/20">
                {paymentData.receiver_address}
              </div>
              <motion.button
                onClick={() => navigator.clipboard.writeText(paymentData.receiver_address)}
                className="mt-3 text-purple-400 hover:text-pink-400 transition font-bold"
                whileHover={{ scale: 1.05 }}
              >
                📋 Copy Address
              </motion.button>
            </div>

            <motion.button
              onClick={() => setStep(2)}
              className="w-full bg-gradient-to-r from-purple-600 via-pink-600 to-purple-600 text-white font-bold py-4 rounded-xl text-lg shadow-lg shadow-purple-500/50"
              whileHover={{ scale: 1.02, boxShadow: "0 20px 60px rgba(168, 85, 247, 0.6)" }}
              whileTap={{ scale: 0.98 }}
            >
              I've Sent the Payment →
            </motion.button>
          </>
        )}

        {step === 2 && (
          <>
            <motion.div className="text-center mb-8">
              <div className="text-6xl mb-4">🔐</div>
              <h1 className="text-4xl font-serif font-bold text-transparent bg-clip-text bg-gradient-to-r from-purple-400 via-pink-400 to-purple-400 mb-2">Confirm Transaction</h1>
              <p className="text-purple-300">Enter your payment details</p>
            </motion.div>
            
            <div className="space-y-6">
              <div>
                <label className="block mb-2 text-purple-300 font-bold">Your Wallet Address</label>
                <motion.input
                  type="text"
                  placeholder="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
                  value={formData.walletAddress}
                  onChange={(e) => setFormData({ ...formData, walletAddress: e.target.value })}
                  className="w-full p-4 rounded-xl border-2 border-purple-500/30 bg-black/50 text-purple-100 placeholder-purple-400/50 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
                  whileFocus={{ scale: 1.01, borderColor: "rgba(168, 85, 247, 0.8)" }}
                />
              </div>

              <div>
                <label className="block mb-2 text-purple-300 font-bold">Amount Sent</label>
                <motion.input
                  type="text"
                  placeholder="0.00"
                  value={formData.amount}
                  onChange={(e) => setFormData({ ...formData, amount: e.target.value })}
                  className="w-full p-4 rounded-xl border-2 border-purple-500/30 bg-black/50 text-purple-100 placeholder-purple-400/50 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
                  whileFocus={{ scale: 1.01, borderColor: "rgba(168, 85, 247, 0.8)" }}
                />
              </div>

              <div>
                <label className="block mb-2 text-purple-300 font-bold">Transaction Hash</label>
                <motion.input
                  type="text"
                  placeholder="0x1234567890abcdef..."
                  value={formData.txHash}
                  onChange={(e) => setFormData({ ...formData, txHash: e.target.value })}
                  className="w-full p-4 rounded-xl border-2 border-purple-500/30 bg-black/50 text-purple-100 placeholder-purple-400/50 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
                  whileFocus={{ scale: 1.01, borderColor: "rgba(168, 85, 247, 0.8)" }}
                />
              </div>

              <motion.button
                onClick={handleConfirm}
                disabled={loading || !formData.walletAddress || !formData.txHash}
                className="w-full bg-gradient-to-r from-purple-600 via-pink-600 to-purple-600 text-white font-bold py-4 rounded-xl text-lg disabled:opacity-50 shadow-lg shadow-purple-500/50"
                whileHover={{ scale: 1.02, boxShadow: "0 20px 60px rgba(168, 85, 247, 0.6)" }}
                whileTap={{ scale: 0.98 }}
              >
                {loading ? "Confirming..." : "Confirm Payment"}
              </motion.button>
            </div>
          </>
        )}

        {step === 3 && (
          <motion.div
            className="text-center"
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            transition={{ type: "spring", stiffness: 200 }}
          >
            <motion.div
              className="text-9xl mb-6"
              animate={{ rotate: [0, 360], scale: [1, 1.2, 1] }}
              transition={{ duration: 1 }}
            >
              ✅
            </motion.div>
            <h1 className="text-5xl font-serif font-bold text-transparent bg-clip-text bg-gradient-to-r from-green-400 via-emerald-400 to-green-400 mb-4">Payment Confirmed!</h1>
            <p className="text-purple-300 mb-8 text-xl">Your booking is complete. Check your email for details.</p>
            <motion.button
              onClick={() => navigate("/flights")}
              className="bg-gradient-to-r from-purple-600 via-pink-600 to-purple-600 text-white font-bold px-10 py-4 rounded-xl text-lg shadow-lg shadow-purple-500/50"
              whileHover={{ scale: 1.05, boxShadow: "0 20px 60px rgba(168, 85, 247, 0.6)" }}
            >
              Book Another Flight
            </motion.button>
          </motion.div>
        )}
      </motion.div>
    </motion.div>
  );
}
