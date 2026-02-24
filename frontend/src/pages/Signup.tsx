import { motion } from "framer-motion";
import { useState } from "react";
import { useNavigate, Link } from "react-router-dom";
import { useAuth } from "../context/AuthContext";
import { useTheme } from "../context/ThemeContext";

export default function Signup() {
  const [formData, setFormData] = useState({ name: "", email: "", password: "", confirmPassword: "" });
  const { login } = useAuth();
  const { theme } = useTheme();
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (formData.password !== formData.confirmPassword) {
      alert("Passwords don't match");
      return;
    }
    await login(formData.email, formData.password);
    navigate("/");
  };

  return (
    <div className={`min-h-screen flex items-center justify-center p-6 ${theme === "dark" ? "bg-gradient-to-br from-luxury-midnight via-luxury-charcoal to-black" : "bg-gradient-to-br from-gray-50 via-white to-blue-50"}`}>
      <motion.div
        className={`glass-effect rounded-3xl p-8 max-w-md w-full border ${theme === "dark" ? "border-luxury-gold/30" : "border-gray-300"}`}
        initial={{ scale: 0.9, opacity: 0, y: 50 }}
        animate={{ scale: 1, opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
      >
        <motion.h1
          className={`text-4xl font-serif font-bold mb-2 text-center ${theme === "dark" ? "text-luxury-gold" : "text-gray-900"}`}
          animate={{ textShadow: theme === "dark" ? ["0 0 20px rgba(212,175,55,0.3)", "0 0 40px rgba(212,175,55,0.6)", "0 0 20px rgba(212,175,55,0.3)"] : "none" }}
          transition={{ duration: 2, repeat: Infinity }}
        >
          Create Account
        </motion.h1>
        <p className={`text-center mb-8 ${theme === "dark" ? "text-luxury-platinum" : "text-gray-600"}`}>
          Join the elite travel experience
        </p>

        <form onSubmit={handleSubmit} className="space-y-5">
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

          <motion.input
            type="password"
            placeholder="Password"
            value={formData.password}
            onChange={(e) => setFormData({ ...formData, password: e.target.value })}
            className={`w-full p-4 rounded-xl border focus:outline-none focus:ring-2 ${theme === "dark" ? "bg-luxury-charcoal/50 border-luxury-gold/20 text-luxury-ivory placeholder-luxury-platinum/50 focus:ring-luxury-gold" : "bg-white border-gray-300 text-gray-900 placeholder-gray-400 focus:ring-blue-500"}`}
            whileFocus={{ scale: 1.02 }}
            required
          />

          <motion.input
            type="password"
            placeholder="Confirm Password"
            value={formData.confirmPassword}
            onChange={(e) => setFormData({ ...formData, confirmPassword: e.target.value })}
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
            Create Account
          </motion.button>
        </form>

        <p className={`text-center mt-6 ${theme === "dark" ? "text-luxury-platinum" : "text-gray-600"}`}>
          Already have an account?{" "}
          <Link to="/login" className={`font-bold ${theme === "dark" ? "text-luxury-gold hover:text-luxury-platinum" : "text-blue-600 hover:text-blue-800"}`}>
            Login
          </Link>
        </p>
      </motion.div>
    </div>
  );
}
