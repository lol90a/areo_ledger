import { motion } from "framer-motion";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../context/AuthContext";
import { useTheme } from "../context/ThemeContext";

export default function AdminLogin() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [code, setCode] = useState("");
  const { login } = useAuth();
  const { theme } = useTheme();
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email.includes("admin") || code !== "1234") {
      alert("Invalid admin credentials");
      return;
    }
    await login(email, password);
    navigate("/admin");
  };

  return (
    <div className={`min-h-screen flex items-center justify-center p-6 relative overflow-hidden ${theme === "dark" ? "bg-black" : "bg-gray-900"}`}>
      <div className="absolute inset-0 bg-gradient-to-br from-red-900/20 via-black to-luxury-gold/20" />
      
      <motion.div
        className="glass-effect rounded-3xl p-10 max-w-md w-full border-2 border-red-600/50 relative z-10"
        initial={{ scale: 0.8, opacity: 0, rotateY: 180 }}
        animate={{ scale: 1, opacity: 1, rotateY: 0 }}
        transition={{ duration: 0.8, type: "spring" }}
      >
        <motion.div
          className="text-center mb-8"
          animate={{ scale: [1, 1.05, 1] }}
          transition={{ duration: 2, repeat: Infinity }}
        >
          <div className="text-6xl mb-4">🔐</div>
          <h1 className="text-4xl font-serif font-bold text-red-500 mb-2">Admin Access</h1>
          <p className="text-red-300">Authorized Personnel Only</p>
        </motion.div>

        <form onSubmit={handleSubmit} className="space-y-6">
          <motion.input
            type="email"
            placeholder="Admin Email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            className="w-full p-4 rounded-xl border-2 border-red-600/30 bg-black/50 text-red-100 placeholder-red-400/50 focus:outline-none focus:ring-2 focus:ring-red-500"
            whileFocus={{ scale: 1.02, borderColor: "rgba(220, 38, 38, 0.8)" }}
            required
          />

          <motion.input
            type="password"
            placeholder="Password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            className="w-full p-4 rounded-xl border-2 border-red-600/30 bg-black/50 text-red-100 placeholder-red-400/50 focus:outline-none focus:ring-2 focus:ring-red-500"
            whileFocus={{ scale: 1.02, borderColor: "rgba(220, 38, 38, 0.8)" }}
            required
          />

          <motion.input
            type="text"
            placeholder="Admin Code"
            value={code}
            onChange={(e) => setCode(e.target.value)}
            className="w-full p-4 rounded-xl border-2 border-red-600/30 bg-black/50 text-red-100 placeholder-red-400/50 focus:outline-none focus:ring-2 focus:ring-red-500"
            whileFocus={{ scale: 1.02, borderColor: "rgba(220, 38, 38, 0.8)" }}
            required
          />

          <motion.button
            type="submit"
            className="w-full bg-gradient-to-r from-red-600 to-red-800 text-white font-bold py-4 rounded-xl text-lg"
            whileHover={{ scale: 1.02, boxShadow: "0 10px 40px rgba(220, 38, 38, 0.6)" }}
            whileTap={{ scale: 0.98 }}
          >
            Access Admin Panel
          </motion.button>
        </form>

        <p className="text-center mt-6 text-red-300 text-sm">
          Code: 1234 (Demo)
        </p>
      </motion.div>
    </div>
  );
}
