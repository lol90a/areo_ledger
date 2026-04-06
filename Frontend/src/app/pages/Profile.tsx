import { motion } from "motion/react";
import { User, Shield, Wallet, Bell, Lock, Smartphone, Fingerprint, Activity } from "lucide-react";

export default function Profile() {
  return (
    <div className="max-w-7xl mx-auto px-6 py-8">
      <div className="mb-10">
        <h1 className="text-3xl md:text-4xl font-bold text-white mb-2">Account Settings</h1>
        <p className="text-slate-400">Manage your profile, security preferences, and verified payment methods.</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
        {/* Sidebar Nav */}
        <div className="flex flex-col gap-2 p-4 rounded-3xl bg-[#0A0A0E] border border-white/5 h-fit">
          <button className="flex items-center gap-3 px-4 py-3 rounded-xl bg-amber-500/10 text-amber-500 font-medium w-full text-left transition-colors">
            <User className="w-5 h-5" /> Personal Details
          </button>
          <button className="flex items-center gap-3 px-4 py-3 rounded-xl text-slate-400 hover:bg-white/5 hover:text-white font-medium w-full text-left transition-colors">
            <Shield className="w-5 h-5" /> Security
          </button>
          <button className="flex items-center gap-3 px-4 py-3 rounded-xl text-slate-400 hover:bg-white/5 hover:text-white font-medium w-full text-left transition-colors">
            <Wallet className="w-5 h-5" /> Payment Methods
          </button>
          <button className="flex items-center gap-3 px-4 py-3 rounded-xl text-slate-400 hover:bg-white/5 hover:text-white font-medium w-full text-left transition-colors">
            <Bell className="w-5 h-5" /> Notifications
          </button>
        </div>

        {/* Content Area */}
        <div className="lg:col-span-3 space-y-6">
          <motion.div 
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className="p-8 rounded-3xl bg-[#0A0A0E] border border-white/5"
          >
            <h2 className="text-xl font-bold text-white mb-6">Profile Information</h2>
            
            <div className="flex flex-col md:flex-row gap-8 items-start mb-8 pb-8 border-b border-white/5">
              <div className="w-24 h-24 rounded-full bg-gradient-to-tr from-amber-600 to-amber-400 flex items-center justify-center text-3xl font-bold text-black shadow-[0_0_20px_rgba(245,158,11,0.3)]">
                JD
              </div>
              <div className="flex-1 w-full space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label className="text-xs uppercase tracking-wider font-semibold text-slate-500 mb-2 block">First Name</label>
                    <input type="text" className="w-full bg-black/50 border border-white/10 rounded-xl px-4 py-3 text-white outline-none focus:border-amber-500" defaultValue="John" />
                  </div>
                  <div>
                    <label className="text-xs uppercase tracking-wider font-semibold text-slate-500 mb-2 block">Last Name</label>
                    <input type="text" className="w-full bg-black/50 border border-white/10 rounded-xl px-4 py-3 text-white outline-none focus:border-amber-500" defaultValue="Doe" />
                  </div>
                  <div className="md:col-span-2">
                    <label className="text-xs uppercase tracking-wider font-semibold text-slate-500 mb-2 block">Email Address (Verified)</label>
                    <input type="email" className="w-full bg-black/50 border border-white/10 rounded-xl px-4 py-3 text-white outline-none focus:border-amber-500" defaultValue="john.doe@example.com" />
                  </div>
                  <div className="md:col-span-2">
                    <label className="text-xs uppercase tracking-wider font-semibold text-slate-500 mb-2 block">Primary Phone</label>
                    <input type="tel" className="w-full bg-black/50 border border-white/10 rounded-xl px-4 py-3 text-white outline-none focus:border-amber-500" defaultValue="+1 (555) 123-4567" />
                  </div>
                </div>
              </div>
            </div>

            <div className="flex justify-end gap-4">
              <button className="px-6 py-3 rounded-xl border border-white/10 text-white font-medium hover:bg-white/5 transition-colors">
                Cancel
              </button>
              <button className="px-6 py-3 rounded-xl bg-amber-500 hover:bg-amber-400 text-black font-semibold shadow-[0_0_15px_rgba(245,158,11,0.3)] transition-all">
                Save Changes
              </button>
            </div>
          </motion.div>

          {/* Verification Status */}
          <motion.div 
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1 }}
            className="p-8 rounded-3xl bg-gradient-to-r from-[#0A0A0E] to-amber-900/10 border border-amber-500/20"
          >
            <div className="flex items-start gap-6">
              <div className="w-12 h-12 rounded-xl bg-amber-500/20 flex items-center justify-center flex-shrink-0">
                <Shield className="w-6 h-6 text-amber-500" />
              </div>
              <div>
                <h3 className="text-lg font-bold text-white mb-2">Institutional KYC/AML Verified</h3>
                <p className="text-sm text-slate-400 mb-4 leading-relaxed">Your account has been fully verified for high-value asset acquisitions and unlimited private charters. Escrow services are enabled.</p>
                <div className="flex gap-4">
                  <span className="flex items-center gap-1.5 text-xs font-semibold text-emerald-500">
                    <Activity className="w-4 h-4" /> Identity Verified
                  </span>
                  <span className="flex items-center gap-1.5 text-xs font-semibold text-emerald-500">
                    <Fingerprint className="w-4 h-4" /> Biometrics Linked
                  </span>
                </div>
              </div>
            </div>
          </motion.div>
        </div>
      </div>
    </div>
  );
}
