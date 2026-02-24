import { motion, AnimatePresence } from "framer-motion";
import { useState, useEffect } from "react";
import { searchFlights } from "../services/api";
import FlightCard from "../components/FlightCard";
import { useTheme } from "../context/ThemeContext";

export default function Flights() {
  const [flights, setFlights] = useState<any[]>([]);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(true);
  const [filters, setFilters] = useState({ type: "all", priceRange: "all", date: "" });
  const { theme } = useTheme();

  useEffect(() => {
    handleSearch();
  }, []);

  const handleSearch = async () => {
    setLoading(true);
    try {
      const res = await searchFlights(query);
      setFlights(res.data);
    } catch (error) {
      console.error("Error fetching flights:", error);
    } finally {
      setLoading(false);
    }
  };

  const filteredFlights = flights.filter((f) => {
    if (filters.type !== "all" && f.type !== filters.type) return false;
    if (filters.priceRange === "low" && f.price_usd > 5000) return false;
    if (filters.priceRange === "high" && f.price_usd < 5000) return false;
    if (filters.date && !f.departure_time.includes(filters.date)) return false;
    return true;
  });

  return (
    <motion.div
      className={`p-6 max-w-7xl mx-auto min-h-screen ${theme === "dark" ? "" : "bg-gray-50"}`}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.6 }}
    >
      <motion.h2
        className={`text-5xl font-serif font-bold mb-8 text-center ${theme === "dark" ? "text-luxury-gold" : "text-gray-900"}`}
        initial={{ y: -30, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ duration: 0.6 }}
      >
        Find Your Perfect Flight
      </motion.h2>

      <motion.div
        className="mb-8 max-w-4xl mx-auto"
        initial={{ scale: 0.9, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ delay: 0.2, duration: 0.5 }}
      >
        <div className="flex gap-4 mb-6">
          <motion.input
            type="text"
            placeholder="Search destination (e.g., Paris, Dubai, New York)"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
            className={`flex-grow p-4 rounded-xl border focus:outline-none focus:ring-2 ${theme === "dark" ? "bg-luxury-charcoal/50 border-luxury-gold/30 text-luxury-ivory placeholder-luxury-platinum/60 focus:ring-luxury-gold" : "bg-white border-gray-300 text-gray-900 placeholder-gray-400 focus:ring-blue-500"}`}
            whileFocus={{ scale: 1.01 }}
          />
          <motion.button
            onClick={handleSearch}
            className={`px-8 py-4 rounded-xl font-bold ${theme === "dark" ? "bg-gradient-to-r from-luxury-gold to-luxury-platinum text-luxury-midnight" : "bg-gradient-to-r from-blue-600 to-blue-800 text-white"}`}
            whileHover={{ scale: 1.05, boxShadow: "0 10px 30px rgba(212,175,55,0.4)" }}
            whileTap={{ scale: 0.95 }}
          >
            Search
          </motion.button>
        </div>

        <div className={`glass-effect rounded-2xl p-6 border ${theme === "dark" ? "border-luxury-gold/20" : "border-gray-300"}`}>
          <h3 className={`font-bold mb-4 ${theme === "dark" ? "text-luxury-gold" : "text-gray-900"}`}>Filters</h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className={`block mb-2 text-sm ${theme === "dark" ? "text-luxury-platinum" : "text-gray-600"}`}>Flight Type</label>
              <select
                value={filters.type}
                onChange={(e) => setFilters({ ...filters, type: e.target.value })}
                className={`w-full p-3 rounded-xl border focus:outline-none focus:ring-2 ${theme === "dark" ? "bg-luxury-charcoal/50 border-luxury-gold/20 text-luxury-ivory focus:ring-luxury-gold" : "bg-white border-gray-300 text-gray-900 focus:ring-blue-500"}`}
              >
                <option value="all">All Flights</option>
                <option value="commercial">Commercial</option>
                <option value="private">Private Jets</option>
              </select>
            </div>

            <div>
              <label className={`block mb-2 text-sm ${theme === "dark" ? "text-luxury-platinum" : "text-gray-600"}`}>Price Range</label>
              <select
                value={filters.priceRange}
                onChange={(e) => setFilters({ ...filters, priceRange: e.target.value })}
                className={`w-full p-3 rounded-xl border focus:outline-none focus:ring-2 ${theme === "dark" ? "bg-luxury-charcoal/50 border-luxury-gold/20 text-luxury-ivory focus:ring-luxury-gold" : "bg-white border-gray-300 text-gray-900 focus:ring-blue-500"}`}
              >
                <option value="all">All Prices</option>
                <option value="low">Under $5,000</option>
                <option value="high">$5,000+</option>
              </select>
            </div>

            <div>
              <label className={`block mb-2 text-sm ${theme === "dark" ? "text-luxury-platinum" : "text-gray-600"}`}>Departure Date</label>
              <input
                type="date"
                value={filters.date}
                onChange={(e) => setFilters({ ...filters, date: e.target.value })}
                className={`w-full p-3 rounded-xl border focus:outline-none focus:ring-2 ${theme === "dark" ? "bg-luxury-charcoal/50 border-luxury-gold/20 text-luxury-ivory focus:ring-luxury-gold" : "bg-white border-gray-300 text-gray-900 focus:ring-blue-500"}`}
              />
            </div>
          </div>
        </div>
      </motion.div>

      <AnimatePresence mode="wait">
        {loading ? (
          <motion.div
            className={`text-center py-20 ${theme === "dark" ? "text-luxury-platinum" : "text-gray-600"}`}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
          >
            <motion.div
              className={`inline-block w-16 h-16 border-4 rounded-full ${theme === "dark" ? "border-luxury-gold/30 border-t-luxury-gold" : "border-gray-300 border-t-blue-600"}`}
              animate={{ rotate: 360 }}
              transition={{ duration: 1, repeat: Infinity, ease: "linear" }}
            />
            <p className="mt-4 text-xl">Loading flights...</p>
          </motion.div>
        ) : (
          <motion.div
            className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.5 }}
          >
            {filteredFlights.length > 0 ? (
              filteredFlights.map((flight, i) => (
                <motion.div
                  key={flight.id}
                  initial={{ opacity: 0, y: 50 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: i * 0.1, duration: 0.5 }}
                >
                  <FlightCard flight={flight} />
                </motion.div>
              ))
            ) : (
              <motion.div
                className={`col-span-full text-center py-20 ${theme === "dark" ? "text-luxury-platinum" : "text-gray-600"}`}
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
              >
                <p className="text-2xl">No flights found. Try adjusting your filters.</p>
              </motion.div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}
