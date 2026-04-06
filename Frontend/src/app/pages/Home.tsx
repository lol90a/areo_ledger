import { Link } from "react-router";
import { motion } from "motion/react";
import { Plane, ChevronRight, Lock, Hexagon } from "lucide-react";

export function Home() {
  return (
    <div className="min-h-screen bg-[#05050A] text-white overflow-hidden relative selection:bg-amber-500/30">
      {/* Background Image with Overlay */}
      <div 
        className="absolute inset-0 z-0 bg-cover bg-center"
        style={{ backgroundImage: 'url(https://images.unsplash.com/photo-1768346564233-d71f37bd19b6?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3w3Nzg4Nzd8MHwxfHNlYXJjaHwxfHxwcml2YXRlJTIwamV0JTIwbHV4dXJ5JTIwaW50ZXJpb3J8ZW58MXx8fHwxNzc1MDQzOTQwfDA&ixlib=rb-4.1.0&q=80&w=1920)' }}
      >
        <div className="absolute inset-0 bg-gradient-to-b from-[#05050A]/90 via-[#05050A]/70 to-[#05050A] backdrop-blur-[2px]"></div>
      </div>

      {/* Top Navbar */}
      <nav className="relative z-10 flex items-center justify-between px-8 py-6 max-w-7xl mx-auto">
        <Link to="/" className="flex items-center gap-3 group">
          <div className="w-10 h-10 rounded-full bg-gradient-to-tr from-amber-500 to-amber-200 flex items-center justify-center shadow-[0_0_20px_rgba(245,158,11,0.4)] group-hover:shadow-[0_0_30px_rgba(245,158,11,0.6)] transition-shadow">
            <Plane className="w-5 h-5 text-black fill-black transform -rotate-45" />
          </div>
          <span className="text-2xl font-bold tracking-wide text-white">Aero<span className="text-amber-400">Ledger</span></span>
        </Link>
        <div className="flex items-center gap-8">
          <Link to="/dashboard" className="text-sm font-medium text-slate-300 hover:text-white transition-colors tracking-widest uppercase">Platform</Link>
          <a href="#" className="text-sm font-medium text-slate-300 hover:text-white transition-colors tracking-widest uppercase">About</a>
          <a href="#" className="text-sm font-medium text-slate-300 hover:text-white transition-colors tracking-widest uppercase">Security</a>
          <Link to="/dashboard" className="px-6 py-2.5 rounded-full border border-amber-500/50 text-amber-400 hover:bg-amber-500 hover:text-black transition-all duration-300 font-semibold tracking-wide shadow-[0_0_15px_rgba(245,158,11,0.15)] hover:shadow-[0_0_25px_rgba(245,158,11,0.4)]">
            Connect Wallet
          </Link>
        </div>
      </nav>

      {/* Hero Content */}
      <main className="relative z-10 flex flex-col items-center justify-center min-h-[calc(100vh-100px)] text-center px-4 max-w-5xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, ease: "easeOut" }}
          className="inline-flex items-center gap-2 px-4 py-1.5 rounded-full bg-white/5 border border-white/10 backdrop-blur-md mb-8"
        >
          <Lock className="w-4 h-4 text-amber-400" />
          <span className="text-xs font-semibold tracking-widest uppercase text-amber-200">Web3 Luxury Protocol</span>
        </motion.div>
        
        <motion.h1
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.2, ease: "easeOut" }}
          className="text-6xl md:text-8xl font-bold tracking-tighter mb-6 bg-clip-text text-transparent bg-gradient-to-b from-white via-slate-200 to-slate-500 drop-shadow-2xl"
        >
          Book Flights & Assets <br/>
          <span className="bg-clip-text text-transparent bg-gradient-to-r from-amber-200 via-amber-400 to-amber-600">With Crypto.</span>
        </motion.h1>
        
        <motion.p
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.4, ease: "easeOut" }}
          className="text-lg md:text-xl text-slate-400 max-w-2xl mb-12 font-light leading-relaxed"
        >
          The world's first decentralized platform for private aviation, high-end real estate, and luxury assets. Secure, anonymous, and borderless.
        </motion.p>
        
        <motion.div
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.8, delay: 0.6, ease: "easeOut" }}
          className="flex items-center gap-6"
        >
          <Link 
            to="/dashboard"
            className="group relative px-8 py-4 bg-amber-500 text-black font-bold tracking-wider uppercase rounded-full overflow-hidden shadow-[0_0_30px_rgba(245,158,11,0.3)] hover:shadow-[0_0_50px_rgba(245,158,11,0.6)] transition-all flex items-center gap-2"
          >
            <span className="relative z-10">Start Booking</span>
            <ChevronRight className="w-5 h-5 relative z-10 group-hover:translate-x-1 transition-transform" />
            <div className="absolute inset-0 bg-gradient-to-r from-amber-400 to-yellow-300 opacity-0 group-hover:opacity-100 transition-opacity"></div>
          </Link>
          
          <Link 
            to="/assets"
            className="px-8 py-4 bg-white/5 backdrop-blur-md border border-white/10 text-white font-bold tracking-wider uppercase rounded-full hover:bg-white/10 transition-colors flex items-center gap-2"
          >
            <Hexagon className="w-5 h-5 text-amber-400" />
            Explore Assets
          </Link>
        </motion.div>
      </main>
    </div>
  );
}
