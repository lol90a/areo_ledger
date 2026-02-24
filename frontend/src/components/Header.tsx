import { motion } from "framer-motion";
import { useState } from "react";
import { Link } from "react-router-dom";
import { useTheme } from "../context/ThemeContext";
import { useAuth } from "../context/AuthContext";

export default function Header() {
  const { theme, toggleTheme } = useTheme();
  const { user, logout } = useAuth();

  return (
    <motion.header
      className={`backdrop-blur-md px-6 py-4 flex justify-between items-center shadow-lg border-b sticky top-0 z-50 ${theme === "dark" ? "bg-luxury-midnight/90 text-luxury-ivory border-luxury-gold/20" : "bg-white/90 text-gray-900 border-gray-200"}`}
      initial={{ y: -100 }}
      animate={{ y: 0 }}
      transition={{ duration: 0.6, ease: "easeOut" }}
    >
      <Link to="/">
        <motion.div
          className={`text-3xl font-serif font-bold tracking-wide ${theme === "dark" ? "text-luxury-gold" : "text-blue-600"}`}
          whileHover={{ scale: 1.05, textShadow: theme === "dark" ? "0 0 20px rgba(212,175,55,0.6)" : "0 0 20px rgba(37,99,235,0.6)" }}
          transition={{ duration: 0.3 }}
        >
          AeroLedger ✈️
        </motion.div>
      </Link>

      <nav className="flex gap-8 items-center">
        {[{ to: "/flights", label: "Flights" }, { to: "/private-jets", label: "Private Jets" }, user?.role === "admin" && { to: "/admin", label: "Admin" }].filter(Boolean).map((link: any, i) => (
          <Link key={link.to} to={link.to}>
            <motion.div
              className={`relative font-medium ${theme === "dark" ? "text-luxury-ivory" : "text-gray-700"}`}
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: i * 0.1, duration: 0.5 }}
              whileHover={{ color: theme === "dark" ? "#D4AF37" : "#2563eb" }}
            >
              {link.label}
              <motion.div
                className={`absolute -bottom-1 left-0 h-0.5 ${theme === "dark" ? "bg-luxury-gold" : "bg-blue-600"}`}
                initial={{ width: 0 }}
                whileHover={{ width: "100%" }}
                transition={{ duration: 0.3 }}
              />
            </motion.div>
          </Link>
        ))}

        <motion.button
          onClick={toggleTheme}
          className={`p-2 rounded-full ${theme === "dark" ? "bg-luxury-gold/20 text-luxury-gold" : "bg-gray-200 text-gray-700"}`}
          whileHover={{ scale: 1.1, rotate: 180 }}
          whileTap={{ scale: 0.9 }}
        >
          {theme === "dark" ? "☀️" : "🌙"}
        </motion.button>

        {user ? (
          <motion.button
            onClick={logout}
            className={`px-4 py-2 rounded-full font-bold ${theme === "dark" ? "bg-luxury-gold/20 text-luxury-gold hover:bg-luxury-gold/30" : "bg-blue-100 text-blue-600 hover:bg-blue-200"}`}
            whileHover={{ scale: 1.05 }}
          >
            Logout
          </motion.button>
        ) : (
          <Link to="/login">
            <motion.button
              className={`px-4 py-2 rounded-full font-bold ${theme === "dark" ? "bg-gradient-to-r from-luxury-gold to-luxury-platinum text-luxury-midnight" : "bg-gradient-to-r from-blue-600 to-blue-800 text-white"}`}
              whileHover={{ scale: 1.05 }}
            >
              Login
            </motion.button>
          </Link>
        )}
      </nav>
    </motion.header>
  );
}
