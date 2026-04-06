import { motion } from "motion/react";
import { Link, useNavigate } from "react-router";
import { Plane, Calendar, Users, MapPin, Search, ArrowRight, ShieldCheck, ShoppingCart } from "lucide-react";
import { useState } from "react";

import { addFlightToCart } from "../lib/commerce";

const flightOptions = [
  {
    name: "Citation Latitude",
    category: "Midsize Jet",
    pax: 8,
    time: "6h 45m",
    price: "0.24 BTC",
    usd: "$16,500",
    backendFlightId: "00000000-0000-0000-0000-000000000001",
    backendPriceUsd: 16500,
  },
  {
    name: "Challenger 350",
    category: "Super Midsize",
    pax: 9,
    time: "6h 20m",
    price: "0.38 BTC",
    usd: "$26,000",
    recommended: true,
    backendFlightId: "00000000-0000-0000-0000-000000000003",
    backendPriceUsd: 26000,
  },
  {
    name: "Global 6000",
    category: "Ultra Long Range",
    pax: 14,
    time: "5h 50m",
    price: "0.62 BTC",
    usd: "$42,500",
    backendFlightId: "00000000-0000-0000-0000-000000000002",
    backendPriceUsd: 42500,
  },
];

export default function Flights() {
  const [searchState, setSearchState] = useState(false);
  const navigate = useNavigate();

  const addToCart = (jet: (typeof flightOptions)[number]) => {
    addFlightToCart({
      backendFlightId: jet.backendFlightId,
      category: "Flight Booking",
      name: `${jet.name} Charter`,
      subtitle: `${jet.category} ready for London to Dubai dispatch`,
      primaryMeta: "LHR -> DXB",
      secondaryMeta: `Oct 24, 2026 - ${jet.pax} passengers`,
      cryptoPrice: jet.price,
      fiatPrice: jet.usd,
      image: "https://images.unsplash.com/photo-1625513123245-fcb02d69ad12?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3w3Nzg4Nzd8MHwxfHNlYXJjaHwxfHxwcml2YXRlJTIwamV0JTIwaW50ZXJpb3J8ZW58MXx8fHwxNzc1MDMyMzkzfDA&ixlib=rb-4.1.0&q=80&w=1080",
    });
    navigate("/cart?added=flight");
  };

  return (
    <div className="max-w-7xl mx-auto px-6 py-8">
      <div className="mb-12 text-center md:text-left">
        <h1 className="text-4xl md:text-5xl font-bold text-white mb-4">Charter a Flight</h1>
        <p className="text-lg text-slate-400 max-w-2xl">On-demand access to a global fleet of 10,000+ premium aircraft, ready for dispatch. Pay securely with cryptocurrency.</p>
      </div>

      <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="w-full bg-[#0A0A0E] border border-white/10 rounded-3xl p-6 md:p-8 relative z-10 shadow-2xl shadow-black/50">
        <div className="flex gap-4 mb-8 overflow-x-auto hide-scrollbar pb-2">
          {[
            "One Way",
            "Round Trip",
            "Multi-Leg",
          ].map((type, idx) => (
            <button key={idx} className={`px-6 py-2 rounded-full text-sm font-medium transition-all whitespace-nowrap ${idx === 0 ? "bg-amber-500 text-black shadow-[0_0_15px_rgba(245,158,11,0.3)]" : "bg-white/5 text-slate-300 hover:bg-white/10 border border-white/10"}`}>
              {type}
            </button>
          ))}
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
          <div className="relative group"><label className="text-xs uppercase tracking-wider font-semibold text-slate-500 mb-2 block">Origin</label><div className="flex items-center bg-black/50 border border-white/10 rounded-2xl p-4 group-focus-within:border-amber-500/50 transition-colors"><MapPin className="w-5 h-5 text-amber-500 mr-3" /><input type="text" placeholder="City or Airport" className="bg-transparent w-full text-white placeholder-slate-600 outline-none" defaultValue="London (EGGW)" /></div></div>
          <div className="relative group"><label className="text-xs uppercase tracking-wider font-semibold text-slate-500 mb-2 block">Destination</label><div className="flex items-center bg-black/50 border border-white/10 rounded-2xl p-4 group-focus-within:border-amber-500/50 transition-colors"><MapPin className="w-5 h-5 text-amber-500 mr-3" /><input type="text" placeholder="City or Airport" className="bg-transparent w-full text-white placeholder-slate-600 outline-none" defaultValue="Dubai (OMDB)" /></div></div>
          <div className="relative group"><label className="text-xs uppercase tracking-wider font-semibold text-slate-500 mb-2 block">Departure Date</label><div className="flex items-center bg-black/50 border border-white/10 rounded-2xl p-4 group-focus-within:border-amber-500/50 transition-colors"><Calendar className="w-5 h-5 text-amber-500 mr-3" /><input type="text" placeholder="Select Date" className="bg-transparent w-full text-white placeholder-slate-600 outline-none" defaultValue="Oct 24, 2026" /></div></div>
          <div className="relative group"><label className="text-xs uppercase tracking-wider font-semibold text-slate-500 mb-2 block">Passengers</label><div className="flex items-center bg-black/50 border border-white/10 rounded-2xl p-4 group-focus-within:border-amber-500/50 transition-colors"><Users className="w-5 h-5 text-amber-500 mr-3" /><select className="bg-transparent w-full text-white outline-none appearance-none"><option className="bg-black">1-4 Passengers</option><option className="bg-black">5-8 Passengers</option><option className="bg-black">9-14 Passengers</option><option className="bg-black">15+ Passengers</option></select></div></div>
        </div>

        <div className="flex justify-end">
          <button onClick={() => setSearchState(true)} className="w-full md:w-auto px-8 py-4 bg-amber-500 hover:bg-amber-400 text-black font-semibold rounded-2xl flex items-center justify-center gap-2 transition-all shadow-[0_0_20px_rgba(245,158,11,0.3)]">
            <Search className="w-5 h-5" /> Search Availability
          </button>
        </div>
      </motion.div>

      {searchState && (
        <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="mt-16 space-y-6">
          <div className="flex justify-between items-end mb-8 border-b border-white/10 pb-4">
            <div>
              <h2 className="text-2xl font-bold text-white">Available Aircraft</h2>
              <p className="text-slate-400">London to Dubai - Oct 24, 2026 - 6h 45m</p>
            </div>
            <div className="text-sm text-amber-500 font-medium flex items-center gap-2">
              <ShieldCheck className="w-4 h-4" /> 3 Verified Options
            </div>
          </div>

          {flightOptions.map((jet, idx) => (
            <div key={idx} className={`p-6 md:p-8 rounded-3xl bg-[#0A0A0E] border transition-colors flex flex-col md:flex-row gap-8 items-center ${jet.recommended ? "border-amber-500/50 shadow-[0_0_30px_rgba(245,158,11,0.05)]" : "border-white/5 hover:border-white/20"}`}>
              <div className="w-full md:w-48 h-32 rounded-2xl bg-black/50 overflow-hidden relative"><div className="absolute inset-0 bg-cover bg-center opacity-80" style={{ backgroundImage: `url('https://images.unsplash.com/photo-1625513123245-fcb02d69ad12?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3w3Nzg4Nzd8MHwxfHNlYXJjaHwxfHxwcml2YXRlJTIwamV0JTIwaW50ZXJpb3J8ZW58MXx8fHwxNzc1MDMyMzkzfDA&ixlib=rb-4.1.0&q=80&w=1080')` }} /></div>

              <div className="flex-1 w-full">
                {jet.recommended && <span className="text-xs font-bold tracking-wider text-amber-500 uppercase mb-2 block">Recommended Choice</span>}
                <div className="flex justify-between items-start mb-4">
                  <div>
                    <h3 className="text-2xl font-bold text-white">{jet.name}</h3>
                    <p className="text-sm text-slate-400">{jet.category}</p>
                  </div>
                  <div className="text-right">
                    <p className="text-2xl font-bold text-white">{jet.price}</p>
                    <p className="text-sm text-slate-500">~ {jet.usd}</p>
                  </div>
                </div>

                <div className="flex items-center gap-6 text-sm text-slate-300">
                  <span className="flex items-center gap-2"><Users className="w-4 h-4 text-slate-500" /> {jet.pax} Seats</span>
                  <span className="flex items-center gap-2"><Plane className="w-4 h-4 text-slate-500" /> {jet.time}</span>
                </div>
              </div>

              <div className="w-full md:w-auto mt-4 md:mt-0 flex flex-col sm:flex-row gap-3">
                <button type="button" onClick={() => addToCart(jet)} className="w-full px-6 py-4 rounded-xl font-semibold flex items-center justify-center gap-2 transition-all border border-white/10 text-white hover:bg-white/10">
                  <ShoppingCart className="w-4 h-4 text-amber-500" /> Add to Cart
                </button>
                <Link to="/cart" className={`w-full px-8 py-4 rounded-xl font-semibold flex items-center justify-center gap-2 transition-all ${jet.recommended ? "bg-amber-500 hover:bg-amber-400 text-black" : "bg-white/10 hover:bg-white/20 text-white"}`}>
                  Review Cart <ArrowRight className="w-5 h-5" />
                </Link>
              </div>
            </div>
          ))}
        </motion.div>
      )}
    </div>
  );
}
