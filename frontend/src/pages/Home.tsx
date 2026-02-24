import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import { useTheme } from "../context/ThemeContext";

export default function Home() {
  const { theme } = useTheme();

  return (
    <div className={`relative flex flex-col items-center justify-center min-h-[80vh] py-12 overflow-hidden ${theme === "light" ? "bg-gradient-to-br from-gray-50 via-white to-blue-50" : ""}`}>
      <div className="absolute inset-0 overflow-hidden">
        {[...Array(20)].map((_, i) => (
          <motion.div
            key={i}
            className={`absolute w-2 h-2 rounded-full ${theme === "dark" ? "bg-luxury-gold/20" : "bg-blue-500/20"}`}
            initial={{ x: Math.random() * (typeof window !== 'undefined' ? window.innerWidth : 1000), y: Math.random() * (typeof window !== 'undefined' ? window.innerHeight : 1000) }}
            animate={{
              y: [null, Math.random() * (typeof window !== 'undefined' ? window.innerHeight : 1000)],
              x: [null, Math.random() * (typeof window !== 'undefined' ? window.innerWidth : 1000)],
            }}
            transition={{ duration: 10 + Math.random() * 20, repeat: Infinity, ease: "linear" }}
          />
        ))}
      </div>

      <motion.header
        className="text-center mb-12 z-10"
        initial={{ opacity: 0, y: -50 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.8, ease: "easeOut" }}
      >
        <motion.h1
          className={`text-7xl md:text-8xl font-serif font-bold mb-6 tracking-wide ${theme === "dark" ? "text-luxury-gold" : "text-gray-900"}`}
          animate={{ textShadow: theme === "dark" ? ["0 0 20px rgba(212,175,55,0.3)", "0 0 40px rgba(212,175,55,0.6)", "0 0 20px rgba(212,175,55,0.3)"] : "none" }}
          transition={{ duration: 2, repeat: Infinity }}
        >
          AeroLedger
        </motion.h1>
        <motion.p
          className={`text-2xl font-sans mb-2 ${theme === "dark" ? "text-luxury-platinum" : "text-gray-600"}`}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.3, duration: 0.8 }}
        >
          Book flights and buy private jets with crypto
        </motion.p>
        <motion.p
          className={`text-lg font-sans ${theme === "dark" ? "text-luxury-gold" : "text-blue-600"}`}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.5, duration: 0.8 }}
        >
          ₿ Ξ ₮ ◎ Accepted
        </motion.p>
      </motion.header>

      <motion.main
        className="flex flex-wrap justify-center items-center gap-4 z-10"
        initial={{ opacity: 0, scale: 0.8 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ delay: 0.5, duration: 0.6 }}
      >
        <Link to="/flights">
          <motion.button
            className={`relative px-12 py-5 rounded-full shadow-2xl font-sans text-lg overflow-hidden ${theme === "dark" ? "bg-gradient-to-r from-luxury-gold via-luxury-platinum to-luxury-gold bg-[length:200%_100%] text-luxury-midnight" : "bg-gradient-to-r from-blue-600 via-blue-700 to-blue-600 bg-[length:200%_100%] text-white"}`}
            whileHover={{ scale: 1.05, backgroundPosition: "100% 0" }}
            whileTap={{ scale: 0.95 }}
            animate={{ backgroundPosition: ["0% 0", "100% 0", "0% 0"] }}
            transition={{ backgroundPosition: { duration: 3, repeat: Infinity, ease: "linear" } }}
          >
            <motion.span
              className="relative z-10 font-bold"
              whileHover={{ letterSpacing: "0.1em" }}
              transition={{ duration: 0.3 }}
            >
              Book Flights
            </motion.span>
          </motion.button>
        </Link>

        <Link to="/private-jets">
          <motion.button
            className={`relative px-12 py-5 rounded-full shadow-2xl font-sans text-lg overflow-hidden border-2 ${theme === "dark" ? "border-luxury-gold text-luxury-gold hover:bg-luxury-gold hover:text-luxury-midnight" : "border-blue-600 text-blue-600 hover:bg-blue-600 hover:text-white"}`}
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
          >
            <motion.span
              className="relative z-10 font-bold"
              whileHover={{ letterSpacing: "0.1em" }}
              transition={{ duration: 0.3 }}
            >
              Buy Private Jets
            </motion.span>
          </motion.button>
        </Link>

        <motion.div
          className="mt-16 grid grid-cols-3 gap-8 text-center"
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.8, duration: 0.6 }}
        >
          {[{ num: "500+", label: "Destinations" }, { num: "50+", label: "Private Jets" }, { num: "6", label: "Crypto Accepted" }].map((stat, i) => (
            <motion.div
              key={i}
              className={`glass-effect rounded-xl p-6 ${theme === "dark" ? "" : "bg-white/50 border border-gray-200"}`}
              whileHover={{ scale: 1.05, borderColor: theme === "dark" ? "rgba(212,175,55,0.5)" : "rgba(37,99,235,0.5)" }}
            >
              <h3 className={`text-4xl font-bold mb-2 ${theme === "dark" ? "text-luxury-gold" : "text-blue-600"}`}>{stat.num}</h3>
              <p className={theme === "dark" ? "text-luxury-platinum" : "text-gray-600"}>{stat.label}</p>
            </motion.div>
          ))}
        </motion.div>
      </motion.main>
    </div>
  );
}
