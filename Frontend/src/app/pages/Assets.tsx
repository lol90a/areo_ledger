import { useState } from "react";
import { motion } from "motion/react";
import { 
  Diamond, 
  Building2, 
  Car, 
  Plane,
  Filter,
  ArrowRight
} from "lucide-react";
import { PaymentModal } from "../components/PaymentModal";

const categories = [
  { id: "all", name: "All Assets", icon: Diamond },
  { id: "jets", name: "Private Jets", icon: Plane },
  { id: "real-estate", name: "Real Estate", icon: Building2 },
  { id: "cars", name: "Luxury Cars", icon: Car },
];

const assets = [
  {
    id: 1,
    title: "Gulfstream G650ER",
    category: "jets",
    price: "1,250 BTC",
    usd: "$78,500,000",
    image: "https://images.unsplash.com/photo-1768346564233-d71f37bd19b6?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3w3Nzg4Nzd8MHwxfHNlYXJjaHwxfHxwcml2YXRlJTIwamV0JTIwbHV4dXJ5JTIwaW50ZXJpb3J8ZW58MXx8fHwxNzc1MDQzOTQwfDA&ixlib=rb-4.1.0&q=80&w=1080",
    specs: ["19 Passengers", "7,500 nm Range", "Mach 0.925"],
    featured: true
  },
  {
    id: 2,
    title: "Beverly Hills Modern Mansion",
    category: "real-estate",
    price: "450 BTC",
    usd: "$28,000,000",
    image: "https://images.unsplash.com/photo-1629787302738-2c6e9f3dada1?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3w3Nzg4Nzd8MHwxfHNlYXJjaHwxfHxsdXh1cnklMjBtb2Rlcm4lMjBtYW5zaW9uJTIwYXJjaGl0ZWN0dXJlfGVufDF8fHx8MTc3NTA0Mzk0MHww&ixlib=rb-4.1.0&q=80&w=1080",
    specs: ["12,500 sqft", "8 Beds, 12 Baths", "Infinity Pool"],
    featured: false
  },
  {
    id: 3,
    title: "Bugatti Chiron Pur Sport",
    category: "cars",
    price: "60 BTC",
    usd: "$3,800,000",
    image: "https://images.unsplash.com/photo-1672024110512-f7028b49db28?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3w3Nzg4Nzd8MHwxfHNlYXJjaHwxfHxsdXh1cnklMjBzcG9ydHMlMjBjYXIlMjBkYXJrfGVufDF8fHx8MTc3NDk2NTkyMHww&ixlib=rb-4.1.0&q=80&w=1080",
    specs: ["1,500 HP", "0-60 in 2.3s", "W16 Engine"],
    featured: false
  },
  {
    id: 4,
    title: "Bombardier Global 7500",
    category: "jets",
    price: "1,100 BTC",
    usd: "$72,000,000",
    image: "https://images.unsplash.com/photo-1768346564233-d71f37bd19b6?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3w3Nzg4Nzd8MHwxfHNlYXJjaHwxfHxwcml2YXRlJTIwamV0JTIwbHV4dXJ5JTIwaW50ZXJpb3J8ZW58MXx8fHwxNzc1MDQzOTQwfDA&ixlib=rb-4.1.0&q=80&w=1080",
    specs: ["19 Passengers", "7,700 nm Range", "Mach 0.925"],
    featured: false
  }
];

export function Assets() {
  const [activeCategory, setActiveCategory] = useState("all");
  const [selectedAsset, setSelectedAsset] = useState<{ title: string; price: string; image: string; } | null>(null);

  const filteredAssets = activeCategory === "all" 
    ? assets 
    : assets.filter(a => a.category === activeCategory);

  return (
    <div className="p-8 lg:p-12 max-w-[1600px] mx-auto min-h-full">
      <header className="mb-10 flex flex-col md:flex-row md:items-end justify-between gap-6">
        <div>
          <motion.h1 
            initial={{ opacity: 0, y: -20 }}
            animate={{ opacity: 1, y: 0 }}
            className="text-3xl font-bold tracking-wide text-white mb-2"
          >
            Asset Marketplace
          </motion.h1>
          <motion.p 
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.1 }}
            className="text-slate-400 font-medium"
          >
            Acquire world-class assets seamlessly with cryptocurrency.
          </motion.p>
        </div>

        <motion.div 
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          className="flex items-center gap-2 overflow-x-auto pb-2 md:pb-0"
        >
          {categories.map(cat => (
            <button
              key={cat.id}
              onClick={() => setActiveCategory(cat.id)}
              className={`flex items-center gap-2 px-5 py-2.5 rounded-full font-semibold tracking-wide transition-all whitespace-nowrap ${
                activeCategory === cat.id
                  ? 'bg-amber-500 text-black shadow-[0_0_15px_rgba(245,158,11,0.3)]'
                  : 'bg-white/5 border border-white/10 text-slate-300 hover:bg-white/10'
              }`}
            >
              <cat.icon className="w-4 h-4" />
              {cat.name}
            </button>
          ))}
          <button className="p-2.5 rounded-full bg-white/5 border border-white/10 text-slate-300 hover:bg-white/10 ml-2">
            <Filter className="w-5 h-5" />
          </button>
        </motion.div>
      </header>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-8">
        {filteredAssets.map((asset, idx) => (
          <motion.div
            key={asset.id}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: idx * 0.1 }}
            className="group rounded-3xl bg-[#0A0A10] border border-white/10 overflow-hidden hover:border-amber-500/30 transition-all hover:shadow-[0_0_30px_rgba(245,158,11,0.1)] flex flex-col"
          >
            <div className="relative h-64 overflow-hidden">
              <div 
                className="absolute inset-0 bg-cover bg-center transition-transform duration-700 group-hover:scale-105"
                style={{ backgroundImage: `url(${asset.image})` }}
              />
              <div className="absolute inset-0 bg-gradient-to-t from-[#0A0A10] via-transparent to-transparent" />
              
              {asset.featured && (
                <div className="absolute top-4 left-4 px-3 py-1 bg-amber-500 text-black text-xs font-bold uppercase tracking-widest rounded-full shadow-[0_0_15px_rgba(245,158,11,0.5)]">
                  Premium
                </div>
              )}
            </div>

            <div className="p-6 flex-1 flex flex-col">
              <div className="flex-1">
                <p className="text-xs text-amber-500 font-bold uppercase tracking-widest mb-2">{asset.category.replace('-', ' ')}</p>
                <h3 className="text-xl font-bold text-white mb-4 line-clamp-2">{asset.title}</h3>
                
                <ul className="space-y-2 mb-6">
                  {asset.specs.map((spec, i) => (
                    <li key={i} className="text-sm text-slate-400 flex items-center gap-2">
                      <div className="w-1 h-1 rounded-full bg-slate-600" />
                      {spec}
                    </li>
                  ))}
                </ul>
              </div>

              <div className="pt-4 border-t border-white/10 flex items-center justify-between">
                <div>
                  <p className="text-sm text-slate-500 font-medium mb-0.5">{asset.usd}</p>
                  <p className="text-xl font-bold text-amber-400">{asset.price}</p>
                </div>
                <button 
                  onClick={() => setSelectedAsset(asset)}
                  className="w-12 h-12 rounded-full bg-white/5 border border-white/10 flex items-center justify-center group-hover:bg-amber-500 group-hover:border-amber-500 group-hover:text-black text-white transition-all shadow-lg"
                >
                  <ArrowRight className="w-5 h-5 transform group-hover:-rotate-45 transition-transform duration-300" />
                </button>
              </div>
            </div>
          </motion.div>
        ))}
      </div>

      <PaymentModal 
        isOpen={!!selectedAsset} 
        onClose={() => setSelectedAsset(null)}
        asset={selectedAsset}
      />
    </div>
  );
}
