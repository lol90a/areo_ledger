import { motion } from "framer-motion";
import { useEffect, useState } from "react";
import { getAdminData } from "../services/api";
import { useAuth } from "../context/AuthContext";
import { useNavigate } from "react-router-dom";

export default function AdminDashboard() {
  const [data, setData] = useState<any>(null);
  const { user, isAdmin } = useAuth();
  const navigate = useNavigate();

  useEffect(() => {
    if (!isAdmin) {
      navigate("/admin-login");
      return;
    }
    const fetchData = async () => {
      try {
        const res = await getAdminData();
        setData(res.data);
      } catch (err) {
        setData({ bookings: 156, revenue: 2450000, users: 892, flights: 45 });
      }
    };
    fetchData();
  }, [isAdmin]);

  const stats = [
    { label: "Total Bookings", value: data?.bookings || 156, icon: "📊", color: "from-red-500 to-orange-500" },
    { label: "Revenue", value: `$${(data?.revenue || 2450000).toLocaleString()}`, icon: "💰", color: "from-green-500 to-emerald-500" },
    { label: "Active Users", value: data?.users || 892, icon: "👥", color: "from-blue-500 to-cyan-500" },
    { label: "Flights Listed", value: data?.flights || 45, icon: "✈️", color: "from-purple-500 to-pink-500" },
  ];

  return (
    <motion.div
      className="min-h-screen p-6 bg-gradient-to-br from-red-900 via-black to-gray-900"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
    >
      <div className="max-w-7xl mx-auto">
        <motion.div
          className="mb-12 text-center"
          initial={{ y: -50, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
        >
          <h1 className="text-6xl font-serif font-bold text-transparent bg-clip-text bg-gradient-to-r from-red-400 via-orange-400 to-red-400 mb-4">
            Admin Dashboard
          </h1>
          <p className="text-red-300 text-xl">Welcome back, {user?.name || "Administrator"}</p>
        </motion.div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-12">
          {stats.map((stat, i) => (
            <motion.div
              key={i}
              className="bg-black/50 backdrop-blur-md rounded-2xl p-6 border-2 border-red-500/30"
              initial={{ opacity: 0, y: 50 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: i * 0.1 }}
              whileHover={{ scale: 1.05, borderColor: "rgba(239, 68, 68, 0.6)" }}
            >
              <div className="text-5xl mb-4">{stat.icon}</div>
              <h3 className="text-red-300 text-sm mb-2">{stat.label}</h3>
              <p className={`text-4xl font-bold text-transparent bg-clip-text bg-gradient-to-r ${stat.color}`}>
                {stat.value}
              </p>
            </motion.div>
          ))}
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <motion.div
            className="bg-black/50 backdrop-blur-md rounded-2xl p-6 border-2 border-red-500/30"
            initial={{ opacity: 0, x: -50 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.4 }}
          >
            <h2 className="text-2xl font-bold text-red-400 mb-4">Recent Bookings</h2>
            <div className="space-y-3">
              {[1, 2, 3, 4, 5].map((i) => (
                <motion.div
                  key={i}
                  className="bg-red-900/20 p-4 rounded-xl border border-red-500/20"
                  whileHover={{ scale: 1.02, backgroundColor: "rgba(127, 29, 29, 0.3)" }}
                >
                  <div className="flex justify-between items-center">
                    <div>
                      <p className="text-red-100 font-bold">Booking #{1000 + i}</p>
                      <p className="text-red-300 text-sm">User: user{i}@test.com</p>
                    </div>
                    <span className="text-green-400 font-bold">${(Math.random() * 10000 + 1000).toFixed(0)}</span>
                  </div>
                </motion.div>
              ))}
            </div>
          </motion.div>

          <motion.div
            className="bg-black/50 backdrop-blur-md rounded-2xl p-6 border-2 border-red-500/30"
            initial={{ opacity: 0, x: 50 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.4 }}
          >
            <h2 className="text-2xl font-bold text-red-400 mb-4">Quick Actions</h2>
            <div className="space-y-3">
              {[
                { label: "Add New Flight", icon: "➕" },
                { label: "Manage Users", icon: "👥" },
                { label: "View Payments", icon: "💳" },
                { label: "Analytics", icon: "📊" },
                { label: "Settings", icon: "⚙️" },
              ].map((action, i) => (
                <motion.button
                  key={i}
                  className="w-full bg-gradient-to-r from-red-600 to-orange-600 text-white font-bold p-4 rounded-xl text-left flex items-center gap-3"
                  whileHover={{ scale: 1.02, boxShadow: "0 10px 30px rgba(239, 68, 68, 0.5)" }}
                  whileTap={{ scale: 0.98 }}
                >
                  <span className="text-2xl">{action.icon}</span>
                  <span>{action.label}</span>
                </motion.button>
              ))}
            </div>
          </motion.div>
        </div>
      </div>
    </motion.div>
  );
}
