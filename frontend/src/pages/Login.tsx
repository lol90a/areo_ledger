import { motion } from "framer-motion";
import { useState } from "react";
import { useNavigate, Link } from "react-router-dom";
import { useAuth } from "../context/AuthContext";
import { useTheme } from "../context/ThemeContext";

export default function Login() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const { login } = useAuth();
  const { theme } = useTheme();
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await login(email, password);
    navigate("/");
  };

  return (
    <div className={`min-h-screen flex items-center justify-center p-6 ${theme === "dark" ? "bg-gradient-to-br from-luxury-midnight via-luxury-charcoal to-black" : "bg-gradient-to-br from-gray-50 via-white to-blue-50"}`}>
      <motion.div
        className={`glass-effect rounded-3xl p-8 max-w-md w-full border ${theme === "dark" ? "border-luxury-gold/30" : "border-gray-300"}`}
        initial={{ scale: 0.9, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ duration: 0.5 }}
      >
        <motion.h1
          className={`text-4xl font-serif font-bold mb-2 text-center ${theme === "dark" ? "text-luxury-gold" : "text-gray-900"}`}
          animate={{ textShadow: theme === "dark" ? ["0 0 20px rgba(212,175,55,0.3)", "0 0 40px rgba(212,175,55,0.6)", "0 0 20px rgba(212,175,55,0.3)"] : "none" }}
          transition={{ duration: 2, repeat: Infinity }}
        >
          Welcome Back
        </motion.h1>
        <p className={`text-center mb-8 ${theme === "dark" ? "text-luxury-platinum" : "text-gray-600"}`}>
          Login to your account
        </p>

        <form onSubmit={handleSubmit} className="space-y-6">
          <motion.input
            type="email"
            placeholder="Email Address"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            className={`w-full p-4 rounded-xl border focus:outline-none focus:ring-2 ${theme === "dark" ? "bg-luxury-charcoal/50 border-luxury-gold/20 text-luxury-ivory placeholder-luxury-platinum/50 focus:ring-luxury-gold" : "bg-white border-gray-300 text-gray-900 placeholder-gray-400 focus:ring-blue-500"}`}
            whileFocus={{ scale: 1.02 }}
            required
          />

          <motion.input
            type="password"
            placeholder="Password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            className={`w-full p-4 rounded-xl border focus:outline-none focus:ring-2 ${theme === "dark" ? "bg-luxury-charcoal/50 border-luxury-gold/20 text-luxury-ivory placeholder-luxury-platinum/50 focus:ring-luxury-gold" : "bg-white border-gray-300 text-gray-900 placeholder-gray-400 focus:ring-blue-500"}`}
            whileFocus={{ scale: 1.02 }}
            required
          />

          <motion.button
            type="submit"
            className={`w-full py-4 rounded-xl font-bold text-lg ${theme === "dark" ? "bg-gradient-to-r from-luxury-gold to-luxury-platinum text-luxury-midnight" : "bg-gradient-to-r from-blue-600 to-blue-800 text-white"}`}
            whileHover={{ scale: 1.02, boxShadow: "0 10px 40px rgba(212,175,55,0.4)" }}
            whileTap={{ scale: 0.98 }}
          >
            Login
          </motion.button>
        </form>

        <p className={`text-center mt-6 ${theme === "dark" ? "text-luxury-platinum" : "text-gray-600"}`}>
          Don't have an account?{" "}
          <Link to="/signup" className={`font-bold ${theme === "dark" ? "text-luxury-gold hover:text-luxury-platinum" : "text-blue-600 hover:text-blue-800"}`}>
            Sign Up
          </Link>
        </p>

        <div className="mt-6 text-center">
          <Link to="/admin-login" className={`text-sm ${theme === "dark" ? "text-red-400 hover:text-red-300" : "text-red-600 hover:text-red-700"}`}>
            Admin Access →
          </Link>
        </div>
      </motion.div>
    </div>
  );
}
