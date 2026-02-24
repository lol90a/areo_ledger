import { motion } from "framer-motion";
import { useState } from "react";
import { useTheme } from "../context/ThemeContext";
import { Link } from "react-router-dom";

export default function PrivateJets() {
  const { theme } = useTheme();
  const [filter, setFilter] = useState("all");

  const jets = [
    { id: 1, name: "Gulfstream G650", price: 65000000, range: "7000 nm", passengers: 19, speed: "610 mph", image: "🛩️", category: "ultra-long" },
    { id: 2, name: "Bombardier Global 7500", price: 73000000, range: "7700 nm", passengers: 19, speed: "590 mph", image: "✈️", category: "ultra-long" },
    { id: 3, name: "Cessna Citation X", price: 23000000, range: "3460 nm", passengers: 12, speed: "700 mph", image: "🛫", category: "super-mid" },
    { id: 4, name: "Embraer Phenom 300", price: 9500000, range: "2010 nm", passengers: 11, speed: "520 mph", image: "🛬", category: "light" },
    { id: 5, name: "Dassault Falcon 8X", price: 58000000, range: "6450 nm", passengers: 16, speed: "560 mph", image: "🛩️", category: "long-range" },
    { id: 6, name: "Boeing Business Jet", price: 90000000, range: "6000 nm", passengers: 50, speed: "530 mph", image: "✈️", category: "vip-airliner" },
  ];

  const filtered = jets.filter(j => filter === "all" || j.category === filter);

  return (
    <motion.div
      className={`min-h-screen p-6 ${theme === "light" ? "bg-gray-50" : ""}`}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
    >
      <div className="max-w-7xl mx-auto">
        <motion.div
          className="text-center mb-12"
          initial={{ y: -50, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ duration: 0.8 }}
        >
          <h1 className={`text-6xl font-serif font-bold mb-4 ${theme === "dark" ? "text-luxury-gold" : "text-gray-900"}`}>
            Private Jet Collection
          </h1>
          <p className={`text-xl ${theme === "dark" ? "text-luxury-platinum" : "text-gray-600"}`}>
            Own the skies. Purchase your exclusive aircraft.
          </p>
        </motion.div>

        <motion.div
          className={`glass-effect rounded-2xl p-6 mb-8 border ${theme === "dark" ? "border-luxury-gold/20" : "border-gray-300"}`}
          initial={{ scale: 0.95 }}
          animate={{ scale: 1 }}
        >
          <div className="flex flex-wrap gap-3 justify-center">
            {["all", "light", "super-mid", "long-range", "ultra-long", "vip-airliner"].map((cat) => (
              <motion.button
                key={cat}
                onClick={() => setFilter(cat)}
                className={`px-6 py-3 rounded-full font-bold capitalize ${
                  filter === cat
                    ? theme === "dark" ? "bg-luxury-gold text-luxury-midnight" : "bg-blue-600 text-white"
                    : theme === "dark" ? "bg-luxury-charcoal/50 text-luxury-platinum border border-luxury-gold/20" : "bg-white text-gray-700 border border-gray-300"
                }`}
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.95 }}
              >
                {cat.replace("-", " ")}
              </motion.button>
            ))}
          </div>
        </motion.div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          {filtered.map((jet, i) => (
            <motion.div
              key={jet.id}
              className={`rounded-3xl p-6 border-2 overflow-hidden relative ${theme === "dark" ? "bg-gradient-to-br from-luxury-charcoal to-luxury-midnight border-luxury-gold/30" : "bg-white border-gray-200"}`}
              initial={{ opacity: 0, y: 50 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: i * 0.1 }}
              whileHover={{ scale: 1.03, boxShadow: theme === "dark" ? "0 20px 60px rgba(212,175,55,0.4)" : "0 20px 60px rgba(0,0,0,0.1)" }}
            >
              <motion.div
                className="text-8xl text-center mb-4"
                animate={{ rotateY: [0, 360] }}
                transition={{ duration: 20, repeat: Infinity, ease: "linear" }}
              >
                {jet.image}
              </motion.div>

              <h3 className={`text-2xl font-serif font-bold mb-3 ${theme === "dark" ? "text-luxury-gold" : "text-gray-900"}`}>
                {jet.name}
              </h3>

              <div className={`space-y-2 mb-4 text-sm ${theme === "dark" ? "text-luxury-platinum" : "text-gray-600"}`}>
                <div className="flex justify-between">
                  <span>Range:</span>
                  <span className="font-bold">{jet.range}</span>
                </div>
                <div className="flex justify-between">
                  <span>Passengers:</span>
                  <span className="font-bold">{jet.passengers}</span>
                </div>
                <div className="flex justify-between">
                  <span>Max Speed:</span>
                  <span className="font-bold">{jet.speed}</span>
                </div>
              </div>

              <motion.div
                className={`text-4xl font-bold mb-4 ${theme === "dark" ? "text-luxury-gold" : "text-blue-600"}`}
                animate={{ scale: [1, 1.05, 1] }}
                transition={{ duration: 2, repeat: Infinity }}
              >
                ${(jet.price / 1000000).toFixed(1)}M
              </motion.div>

              <Link to={`/booking/jet-${jet.id}`}>
                <motion.button
                  className={`w-full py-3 rounded-full font-bold ${theme === "dark" ? "bg-gradient-to-r from-luxury-gold to-luxury-platinum text-luxury-midnight" : "bg-gradient-to-r from-blue-600 to-blue-800 text-white"}`}
                  whileHover={{ scale: 1.05 }}
                  whileTap={{ scale: 0.95 }}
                >
                  Purchase Now
                </motion.button>
              </Link>
            </motion.div>
          ))}
        </div>
      </div>
    </motion.div>
  );
}
