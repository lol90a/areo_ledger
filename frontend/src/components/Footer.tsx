import { motion } from "framer-motion";
import { useTheme } from "../context/ThemeContext";

export default function Footer() {
  const cryptos = ["₿", "Ξ", "₮", "◎", "$", "B"];
  const { theme } = useTheme();

  return (
    <motion.footer
      className={`backdrop-blur-md text-center py-8 mt-10 border-t font-sans ${theme === "dark" ? "bg-luxury-midnight/90 text-luxury-platinum border-luxury-gold/20" : "bg-white/90 text-gray-600 border-gray-200"}`}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ delay: 0.5, duration: 0.8 }}
    >
      <motion.div
        className="flex justify-center gap-4 mb-4"
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.6, duration: 0.6 }}
      >
        {cryptos.map((crypto, i) => (
          <motion.span
            key={i}
            className={`text-2xl ${theme === "dark" ? "text-luxury-gold" : "text-blue-600"}`}
            whileHover={{ scale: 1.3, rotate: 360 }}
            transition={{ duration: 0.3 }}
          >
            {crypto}
          </motion.span>
        ))}
      </motion.div>
      <motion.p
        className="text-lg"
        whileHover={{ color: theme === "dark" ? "#D4AF37" : "#2563eb" }}
        transition={{ duration: 0.3 }}
      >
        © 2026 AeroLedger. All rights reserved.
      </motion.p>
      <motion.p
        className="text-sm mt-2"
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.7, duration: 0.6 }}
      >
        Book flights & private jets using crypto. BTC • ETH • USDT • USDC • SOL • BNB
      </motion.p>
    </motion.footer>
  );
}
