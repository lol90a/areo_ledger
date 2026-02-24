import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import { useTheme } from "../context/ThemeContext";

interface Flight {
  id: string;
  destination: string;
  origin: string;
  departure_time: string;
  price_usd: number;
  type?: string;
}

export default function FlightCard({ flight }: { flight: Flight }) {
  const { theme } = useTheme();
  const isPrivate = flight.type === "private";

  return (
    <motion.div
      className={`relative rounded-2xl shadow-2xl p-6 flex flex-col justify-between border overflow-hidden group ${theme === "dark" ? "bg-gradient-to-br from-luxury-charcoal to-luxury-midnight border-luxury-gold/20" : "bg-white border-gray-200"}`}
      initial={{ opacity: 0, y: 50 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      whileHover={{ scale: 1.03, boxShadow: theme === "dark" ? "0 20px 60px rgba(212,175,55,0.3)" : "0 20px 60px rgba(0,0,0,0.1)" }}
    >
      <motion.div
        className={`absolute inset-0 ${theme === "dark" ? "bg-gradient-to-r from-transparent via-luxury-gold/10 to-transparent" : "bg-gradient-to-r from-transparent via-blue-500/10 to-transparent"}`}
        initial={{ x: "-100%" }}
        whileHover={{ x: "100%" }}
        transition={{ duration: 0.6 }}
      />

      <div className="relative z-10">
        {isPrivate && (
          <span className={`inline-block mb-2 text-xs font-bold px-3 py-1 rounded-full ${theme === "dark" ? "bg-luxury-gold text-luxury-midnight" : "bg-blue-600 text-white"}`}>
            ✈️ PRIVATE JET
          </span>
        )}
        <motion.h3
          className={`text-2xl font-serif font-bold mb-2 ${theme === "dark" ? "text-luxury-gold" : "text-gray-900"}`}
          whileHover={{ letterSpacing: "0.05em" }}
          transition={{ duration: 0.3 }}
        >
          {flight.origin} → {flight.destination}
        </motion.h3>
        <p className={`mb-2 font-sans ${theme === "dark" ? "text-luxury-platinum" : "text-gray-600"}`}>
          Departure: {new Date(flight.departure_time).toLocaleString()}
        </p>
        <div className="flex gap-2 mb-4">
          <span className={`text-xs px-3 py-1 rounded-full border ${theme === "dark" ? "bg-luxury-gold/20 text-luxury-gold border-luxury-gold/30" : "bg-blue-50 text-blue-600 border-blue-200"}`}>
            ₿ Crypto Accepted
          </span>
        </div>
        <motion.p
          className={`text-3xl font-bold font-sans ${theme === "dark" ? "text-luxury-gold" : "text-blue-600"}`}
          animate={{ scale: [1, 1.05, 1] }}
          transition={{ duration: 2, repeat: Infinity }}
        >
          ${flight.price_usd.toLocaleString()}
        </motion.p>
      </div>

      <Link to={`/booking/${flight.id}`} className="relative z-10">
        <motion.button
          className={`mt-6 w-full text-center px-6 py-3 rounded-full font-bold font-sans ${theme === "dark" ? "bg-gradient-to-r from-luxury-gold to-luxury-platinum text-luxury-midnight" : "bg-gradient-to-r from-blue-600 to-blue-800 text-white"}`}
          whileHover={{ scale: 1.05, boxShadow: "0 10px 30px rgba(212,175,55,0.5)" }}
          whileTap={{ scale: 0.95 }}
        >
          Book Now
        </motion.button>
      </Link>
    </motion.div>
  );
}
