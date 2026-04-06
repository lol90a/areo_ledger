import { motion } from "motion/react";
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";
import { Plane, Bitcoin, ArrowUpRight, ArrowDownRight, Clock, Wallet, History, FileText } from "lucide-react";

const portfolioData = [
  { name: "Jan", value: 1200000 },
  { name: "Feb", value: 1400000 },
  { name: "Mar", value: 1350000 },
  { name: "Apr", value: 1800000 },
  { name: "May", value: 1750000 },
  { name: "Jun", value: 2100000 },
  { name: "Jul", value: 2450000 },
];

const recentTransactions = [
  { id: "TX-9281", type: "Flight Booking", asset: "USDC", amount: "45,000", status: "Completed", date: "Today, 14:30" },
  { id: "TX-9280", type: "Deposit", asset: "BTC", amount: "2.5", status: "Completed", date: "Yesterday, 09:15" },
  { id: "TX-9279", type: "Asset Inquiry", asset: "ETH", amount: "1,200", status: "Pending", date: "Mar 12, 11:20" },
  { id: "TX-9278", type: "Flight Booking", asset: "USDT", amount: "12,500", status: "Completed", date: "Mar 08, 16:45" },
];

export default function Dashboard() {
  return (
    <div className="max-w-7xl mx-auto px-6 py-8">
      <div className="flex flex-col md:flex-row justify-between items-start md:items-end mb-10 gap-4">
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">Welcome Back, Investor</h1>
          <p className="text-slate-400">Here's an overview of your AeroLedger portfolio and flight history.</p>
        </div>
        <div className="flex items-center gap-3">
          <button className="px-5 py-2.5 rounded-full bg-white/5 border border-white/10 text-white font-medium hover:bg-white/10 transition-colors flex items-center gap-2">
            <Wallet className="w-4 h-4" />
            Connect Wallet
          </button>
        </div>
      </div>

      {/* KPI Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-10">
        {[
          { label: "Total Asset Value", value: "$2,450,000", change: "+12.5%", icon: Wallet, up: true },
          { label: "Active Flights", value: "2 Upcoming", change: "Next: 3 Days", icon: Plane, up: true, noColor: true },
          { label: "Crypto Balance", value: "34.2 BTC", change: "-1.2%", icon: Bitcoin, up: false },
          { label: "Total Hours Flown", value: "148.5", change: "+24 this year", icon: Clock, up: true, noColor: true },
        ].map((stat, idx) => (
          <motion.div 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: idx * 0.1 }}
            key={idx} 
            className="p-6 rounded-3xl bg-[#0A0A0E] border border-white/5 relative overflow-hidden group hover:border-amber-500/30 transition-colors"
          >
            <div className="absolute top-0 right-0 p-6 opacity-10 group-hover:opacity-20 transition-opacity">
              <stat.icon className="w-16 h-16 text-amber-500" />
            </div>
            <p className="text-sm font-medium text-slate-400 mb-2 relative z-10">{stat.label}</p>
            <h3 className="text-3xl font-bold text-white mb-4 relative z-10">{stat.value}</h3>
            <div className={`inline-flex items-center gap-1 text-xs font-semibold ${stat.noColor ? 'text-slate-500' : stat.up ? 'text-emerald-500' : 'text-rose-500'} relative z-10`}>
              {!stat.noColor && (stat.up ? <ArrowUpRight className="w-3 h-3" /> : <ArrowDownRight className="w-3 h-3" />)}
              {stat.change}
            </div>
          </motion.div>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Chart */}
        <div className="lg:col-span-2 p-6 rounded-3xl bg-[#0A0A0E] border border-white/5">
          <div className="flex justify-between items-center mb-8">
            <h3 className="text-lg font-bold text-white">Portfolio Value</h3>
            <select className="bg-black/50 border border-white/10 rounded-lg px-3 py-1.5 text-sm text-slate-300 outline-none focus:border-amber-500">
              <option>Past 6 Months</option>
              <option>Past Year</option>
              <option>All Time</option>
            </select>
          </div>
          <div className="h-[300px] w-full">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={portfolioData}>
                <defs>
                  <linearGradient id="colorValue" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#F59E0B" stopOpacity={0.3}/>
                    <stop offset="95%" stopColor="#F59E0B" stopOpacity={0}/>
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#ffffff0a" vertical={false} />
                <XAxis dataKey="name" stroke="#ffffff40" tick={{ fill: '#ffffff80', fontSize: 12 }} axisLine={false} tickLine={false} />
                <YAxis stroke="#ffffff40" tick={{ fill: '#ffffff80', fontSize: 12 }} axisLine={false} tickLine={false} tickFormatter={(val) => `$${val/1000}k`} />
                <Tooltip 
                  contentStyle={{ backgroundColor: '#0A0A0E', borderColor: '#ffffff1a', borderRadius: '12px' }}
                  itemStyle={{ color: '#F59E0B' }}
                />
                <Area type="monotone" dataKey="value" stroke="#F59E0B" strokeWidth={2} fillOpacity={1} fill="url(#colorValue)" />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Recent Activity */}
        <div className="p-6 rounded-3xl bg-[#0A0A0E] border border-white/5 flex flex-col">
          <div className="flex justify-between items-center mb-6">
            <h3 className="text-lg font-bold text-white">Recent Activity</h3>
            <button className="text-amber-500 text-sm hover:text-amber-400">View All</button>
          </div>
          
          <div className="flex-1 flex flex-col gap-4">
            {recentTransactions.map((tx, idx) => (
              <div key={idx} className="flex items-center justify-between p-4 rounded-2xl bg-white/5 hover:bg-white/10 transition-colors cursor-pointer border border-transparent hover:border-white/10">
                <div className="flex items-center gap-4">
                  <div className={`w-10 h-10 rounded-xl flex items-center justify-center ${tx.status === 'Completed' ? 'bg-emerald-500/10' : 'bg-amber-500/10'}`}>
                    {tx.type === 'Flight Booking' ? <Plane className="w-4 h-4 text-slate-300" /> : <FileText className="w-4 h-4 text-slate-300" />}
                  </div>
                  <div>
                    <p className="text-sm font-semibold text-white">{tx.type}</p>
                    <p className="text-xs text-slate-500">{tx.date}</p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="text-sm font-bold text-white">{tx.amount} <span className="text-xs text-slate-400 font-normal">{tx.asset}</span></p>
                  <p className={`text-xs font-medium ${tx.status === 'Completed' ? 'text-emerald-500' : 'text-amber-500'}`}>{tx.status}</p>
                </div>
              </div>
            ))}
          </div>
          
          <button className="w-full mt-6 py-3 rounded-xl border border-white/10 text-sm font-medium text-slate-300 hover:bg-white/5 transition-colors flex items-center justify-center gap-2">
            <History className="w-4 h-4" /> Download Statement
          </button>
        </div>
      </div>
    </div>
  );
}
