import { motion } from "motion/react";
import { Link } from "react-router";
import { Plane, ArrowRight, ShieldCheck, Zap, Globe, Bitcoin } from "lucide-react";

export default function LandingPage() {
  return (
    <div className="flex flex-col w-full">
      {/* Hero Section */}
      <section className="relative w-full h-[85vh] min-h-[600px] flex items-center justify-center overflow-hidden">
        {/* Background Image */}
        <div 
          className="absolute inset-0 z-0 bg-cover bg-center bg-no-repeat opacity-40"
          style={{ backgroundImage: `url('https://images.unsplash.com/photo-1772142228727-db71b13f85e2?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3w3Nzg4Nzd8MHwxfHNlYXJjaHwxfHxwcml2YXRlJTIwamV0JTIwbmlnaHQlMjBsdXh1cnl8ZW58MXx8fHwxNzc1MDQ0NTQ3fDA&ixlib=rb-4.1.0&q=80&w=1080')` }}
        />
        <div className="absolute inset-0 z-0 bg-gradient-to-t from-[#05050A] via-[#05050A]/60 to-transparent" />
        <div className="absolute inset-0 z-0 bg-gradient-to-r from-[#05050A] via-transparent to-[#05050A]" />
        
        <div className="relative z-10 max-w-7xl mx-auto px-6 w-full flex flex-col items-center text-center mt-12">
          <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, ease: "easeOut" }}
            className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-white/5 border border-white/10 backdrop-blur-md mb-8"
          >
            <span className="w-2 h-2 rounded-full bg-amber-500 animate-pulse" />
            <span className="text-xs font-medium tracking-widest text-slate-300 uppercase">Now accepting Web3 Payments</span>
          </motion.div>
          
          <motion.h1 
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, delay: 0.1, ease: "easeOut" }}
            className="text-5xl md:text-7xl lg:text-8xl font-bold tracking-tight text-white mb-6"
          >
            Aviation for the <br className="hidden md:block" />
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-amber-200 via-amber-400 to-amber-600">Crypto Elite</span>
          </motion.h1>
          
          <motion.p 
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, delay: 0.2, ease: "easeOut" }}
            className="max-w-2xl text-lg md:text-xl text-slate-400 mb-10 font-light"
          >
            Book private charters and acquire luxury aircraft, real estate, and vehicles anonymously and securely using BTC, ETH, and stablecoins.
          </motion.p>
          
          <motion.div 
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, delay: 0.3, ease: "easeOut" }}
            className="flex flex-col sm:flex-row gap-4 w-full sm:w-auto"
          >
            <Link 
              to="/flights" 
              className="px-8 py-4 bg-amber-500 hover:bg-amber-400 text-black font-semibold rounded-full flex items-center justify-center gap-2 transition-all shadow-[0_0_20px_rgba(245,158,11,0.3)] hover:shadow-[0_0_30px_rgba(245,158,11,0.5)]"
            >
              <Plane className="w-5 h-5" />
              Book a Flight
            </Link>
            <Link 
              to="/marketplace" 
              className="px-8 py-4 bg-white/5 hover:bg-white/10 border border-white/10 text-white font-medium rounded-full flex items-center justify-center gap-2 transition-all backdrop-blur-md"
            >
              Browse Assets
              <ArrowRight className="w-5 h-5" />
            </Link>
          </motion.div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-24 bg-[#0A0A0E] relative overflow-hidden border-t border-white/5">
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-full max-w-lg h-[1px] bg-gradient-to-r from-transparent via-amber-500/50 to-transparent" />
        
        <div className="max-w-7xl mx-auto px-6">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold text-white mb-4">Institutional-Grade Assets</h2>
            <p className="text-slate-400 max-w-2xl mx-auto">Experience seamless transitions from digital wealth to physical luxury.</p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <div className="p-8 rounded-3xl bg-gradient-to-b from-white/5 to-transparent border border-white/10 backdrop-blur-sm group hover:border-amber-500/30 transition-colors">
              <div className="w-14 h-14 rounded-2xl bg-amber-500/10 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
                <Bitcoin className="w-7 h-7 text-amber-500" />
              </div>
              <h3 className="text-xl font-semibold text-white mb-3">Crypto-Native Settlements</h3>
              <p className="text-slate-400 leading-relaxed">Direct wallet-to-wallet transactions. Bypass traditional banking limits and delays for high-ticket acquisitions.</p>
            </div>
            
            <div className="p-8 rounded-3xl bg-gradient-to-b from-white/5 to-transparent border border-white/10 backdrop-blur-sm group hover:border-amber-500/30 transition-colors">
              <div className="w-14 h-14 rounded-2xl bg-amber-500/10 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
                <ShieldCheck className="w-7 h-7 text-amber-500" />
              </div>
              <h3 className="text-xl font-semibold text-white mb-3">Absolute Privacy</h3>
              <p className="text-slate-400 leading-relaxed">Discreet bookings and anonymous acquisitions backed by zero-knowledge proofs and secure escrow.</p>
            </div>

            <div className="p-8 rounded-3xl bg-gradient-to-b from-white/5 to-transparent border border-white/10 backdrop-blur-sm group hover:border-amber-500/30 transition-colors">
              <div className="w-14 h-14 rounded-2xl bg-amber-500/10 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
                <Globe className="w-7 h-7 text-amber-500" />
              </div>
              <h3 className="text-xl font-semibold text-white mb-3">Global Access</h3>
              <p className="text-slate-400 leading-relaxed">Instant access to over 10,000 vetted aircraft, premium estates, and exotic vehicles worldwide.</p>
            </div>
          </div>
        </div>
      </section>

      {/* Featured Acquisitions (Teaser) */}
      <section className="py-24 bg-[#05050A]">
        <div className="max-w-7xl mx-auto px-6">
          <div className="flex justify-between items-end mb-12">
            <div>
              <h2 className="text-3xl md:text-4xl font-bold text-white mb-4">Featured Acquisitions</h2>
              <p className="text-slate-400">Exclusive assets currently available for purchase via crypto.</p>
            </div>
            <Link to="/marketplace" className="hidden md:flex items-center gap-2 text-amber-500 hover:text-amber-400 font-medium">
              View Marketplace <ArrowRight className="w-4 h-4" />
            </Link>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="group relative rounded-3xl overflow-hidden h-[400px]">
              <div 
                className="absolute inset-0 bg-cover bg-center transition-transform duration-700 group-hover:scale-105"
                style={{ backgroundImage: `url('https://images.unsplash.com/photo-1657409845150-f31d72aff3a0?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3w3Nzg4Nzd8MHwxfHNlYXJjaHwxfHxndWxmc3RyZWFtJTIwamV0fGVufDF8fHx8MTc3NTA0NDU1MHww&ixlib=rb-4.1.0&q=80&w=1080')` }}
              />
              <div className="absolute inset-0 bg-gradient-to-t from-black via-black/40 to-transparent" />
              <div className="absolute bottom-0 left-0 p-6 w-full">
                <div className="flex flex-col gap-1">
                  <span className="text-xs font-bold tracking-wider text-amber-500 uppercase">Heavy Jet</span>
                  <h3 className="text-xl font-bold text-white">Gulfstream G650ER</h3>
                  <p className="text-slate-300 text-sm mb-2">New York • 2022</p>
                  <p className="text-lg font-bold text-white flex items-center gap-1">
                    <Bitcoin className="w-4 h-4 text-amber-500" /> 845.2 BTC
                  </p>
                </div>
              </div>
            </div>

            <div className="group relative rounded-3xl overflow-hidden h-[400px]">
              <div 
                className="absolute inset-0 bg-cover bg-center transition-transform duration-700 group-hover:scale-105"
                style={{ backgroundImage: `url('https://images.unsplash.com/photo-1677553512940-f79af72efd1b?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3w3Nzg4Nzd8MHwxfHNlYXJjaHwxfHxsdXh1cnklMjBwZW50aG91c2UlMjBpbnRlcmlvcnxlbnwxfHx8fDE3NzUyMjY4NDN8MA&ixlib=rb-4.1.0&q=80&w=1080')` }}
              />
              <div className="absolute inset-0 bg-gradient-to-t from-black via-black/40 to-transparent" />
              <div className="absolute bottom-0 left-0 p-6 w-full">
                <div className="flex flex-col gap-1">
                  <span className="text-xs font-bold tracking-wider text-amber-500 uppercase">Real Estate</span>
                  <h3 className="text-xl font-bold text-white">Skyline Penthouse</h3>
                  <p className="text-slate-300 text-sm mb-2">Dubai • 8,500 sq ft</p>
                  <p className="text-lg font-bold text-white flex items-center gap-1">
                    <Bitcoin className="w-4 h-4 text-amber-500" /> 420.5 BTC
                  </p>
                </div>
              </div>
            </div>

            <div className="group relative rounded-3xl overflow-hidden h-[400px]">
              <div 
                className="absolute inset-0 bg-cover bg-center transition-transform duration-700 group-hover:scale-105"
                style={{ backgroundImage: `url('https://images.unsplash.com/photo-1668037069509-ba4c569475b1?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3w3Nzg4Nzd8MHwxfHNlYXJjaHwxfHxmZXJyYXJpJTIwc3VwZXJjYXIlMjBuaWdodHxlbnwxfHx8fDE3NzUzMTA4MTZ8MA&ixlib=rb-4.1.0&q=80&w=1080')` }}
              />
              <div className="absolute inset-0 bg-gradient-to-t from-black via-black/40 to-transparent" />
              <div className="absolute bottom-0 left-0 p-6 w-full">
                <div className="flex flex-col gap-1">
                  <span className="text-xs font-bold tracking-wider text-amber-500 uppercase">Supercar</span>
                  <h3 className="text-xl font-bold text-white">SF90 Stradale</h3>
                  <p className="text-slate-300 text-sm mb-2">Monaco • 2023</p>
                  <p className="text-lg font-bold text-white flex items-center gap-1">
                    <Bitcoin className="w-4 h-4 text-amber-500" /> 8.5 BTC
                  </p>
                </div>
              </div>
            </div>
          </div>
          
          <div className="mt-8 text-center md:hidden">
            <Link to="/marketplace" className="inline-flex items-center gap-2 text-amber-500 font-medium">
              View All Inventory <ArrowRight className="w-4 h-4" />
            </Link>
          </div>
        </div>
      </section>
    </div>
  );
}
