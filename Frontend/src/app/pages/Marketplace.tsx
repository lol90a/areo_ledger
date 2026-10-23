import { motion } from "motion/react";
import { Link, useNavigate } from "react-router";
import { Filter, Search, Compass, Fuel, Users, ArrowRight, Star, Home, Car, Plane, BedDouble, Bath, Maximize, Zap, Gauge, ShoppingCart, Boxes } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { ASSET_DATA } from "../data/assets";
import { fetchAssets, type AssetCatalogItem } from "../lib/backend";
import { addBackendAssetToCart } from "../lib/commerce";

const FALLBACK_IMAGES = {
  Aircraft: "https://images.unsplash.com/photo-1540962351504-03099e0a754b?auto=format&fit=crop&w=1080&q=80",
  "Real Estate": "https://images.unsplash.com/photo-1600607687920-4e2a09cf159d?auto=format&fit=crop&w=1080&q=80",
  Vehicles: "https://images.unsplash.com/photo-1592198084033-aade902d1aae?auto=format&fit=crop&w=1080&q=80",
} as const;

const ASSET_TYPE_FALLBACK_IMAGE = "https://images.unsplash.com/photo-1540962351504-03099e0a754b?auto=format&fit=crop&w=1080&q=80";

function formatUsd(value: number) {
  return `$${value.toLocaleString(undefined, { maximumFractionDigits: 0 })}`;
}

// Live marketplace grid backed by GET /api/assets. Ids and prices are
// server-authoritative so bookings settle against the real records.
function LiveMarketplaceGrid({ assets }: { assets: AssetCatalogItem[] }) {
  const navigate = useNavigate();
  const [filter, setFilter] = useState("All");
  const [query, setQuery] = useState("");
  const [added, setAdded] = useState<string | null>(null);

  const types = useMemo(() => {
    const unique = Array.from(new Set(assets.map((asset) => asset.type)));
    return ["All", ...unique];
  }, [assets]);

  const filtered = assets.filter((asset) => {
    const matchesType = filter === "All" || asset.type === filter;
    const haystack = `${asset.name} ${asset.type} ${asset.description ?? ""}`.toLowerCase();
    return matchesType && haystack.includes(query.trim().toLowerCase());
  });

  const handleAdd = (asset: AssetCatalogItem) => {
    addBackendAssetToCart(asset);
    setAdded(asset.id);
    navigate("/cart?added=asset");
  };

  return (
    <div className="max-w-7xl mx-auto px-6 py-8">
      <div className="flex flex-col md:flex-row justify-between items-start md:items-end mb-12 gap-6">
        <div>
          <h1 className="text-4xl md:text-5xl font-bold text-white mb-4">Luxury Marketplace</h1>
          <p className="text-lg text-slate-400 max-w-2xl">Acquire tokenized shares in the world's finest aircraft, real estate, and vehicles. Prices and availability are pulled live from the AeroLedger backend.</p>
        </div>
        <div className="relative flex-1 md:w-64 w-full md:max-w-xs">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
          <input
            type="text"
            placeholder="Search assets..."
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            className="w-full pl-11 pr-4 py-3 rounded-full bg-white/5 border border-white/10 text-white placeholder-slate-500 focus:outline-none focus:border-amber-500 transition-colors"
          />
        </div>
      </div>

      <div className="flex overflow-x-auto pb-4 mb-8 gap-3 hide-scrollbar">
        {types.map((type) => (
          <button
            key={type}
            onClick={() => setFilter(type)}
            className={`whitespace-nowrap px-6 py-2 rounded-full text-sm font-medium transition-all capitalize ${filter === type ? "bg-amber-500 text-black shadow-[0_0_15px_rgba(245,158,11,0.3)]" : "bg-white/5 text-slate-300 hover:bg-white/10 border border-white/10"}`}
          >
            {type}
          </button>
        ))}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        {filtered.map((asset, idx) => (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: idx * 0.08 }}
            key={asset.id}
            className="group flex flex-col rounded-3xl bg-[#0A0A0E] border border-white/5 overflow-hidden hover:border-amber-500/30 transition-all duration-500"
          >
            <div className="relative h-64 overflow-hidden">
              <div
                className="absolute inset-0 bg-cover bg-center transition-transform duration-700 group-hover:scale-105"
                style={{ backgroundImage: `url('${asset.image ?? ASSET_TYPE_FALLBACK_IMAGE}')` }}
              />
              <div className="absolute inset-0 bg-gradient-to-t from-[#0A0A0E] via-transparent to-transparent" />
              <div className="absolute top-4 right-4 px-3 py-1 rounded-full bg-black/50 backdrop-blur-md border border-white/10 flex items-center gap-1.5">
                <Boxes className="w-3 h-3 text-slate-300" />
                <span className="text-xs font-bold tracking-wider text-slate-200 uppercase">{asset.type}</span>
              </div>
            </div>

            <div className="p-6 md:p-8 flex-1 flex flex-col">
              <div className="flex justify-between items-start mb-4">
                <div>
                  <span className="text-xs font-medium text-amber-500 uppercase tracking-wider mb-1 block">{asset.type}</span>
                  <h3 className="text-2xl font-bold text-white">{asset.name}</h3>
                </div>
                <div className="text-right">
                  <p className="text-xl font-bold text-white">{formatUsd(asset.token_price)}</p>
                  <p className="text-sm text-slate-500">per token</p>
                </div>
              </div>

              {asset.description && <p className="text-sm text-slate-400 mb-6">{asset.description}</p>}

              <div className="grid grid-cols-2 gap-4 py-6 mt-auto border-t border-b border-white/5">
                <div className="flex flex-col gap-1">
                  <span className="text-xs uppercase tracking-wider text-slate-500">Total value</span>
                  <span className="text-sm font-semibold text-slate-200">{formatUsd(asset.total_value)}</span>
                </div>
                <div className="flex flex-col gap-1">
                  <span className="text-xs uppercase tracking-wider text-slate-500">Available tokens</span>
                  <span className="text-sm font-semibold text-slate-200">{asset.available_tokens.toLocaleString()} / {asset.total_tokens.toLocaleString()}</span>
                </div>
              </div>

              <div className="mt-6 flex items-center justify-end">
                <button
                  type="button"
                  onClick={() => handleAdd(asset)}
                  className="flex items-center gap-2 px-5 py-2.5 rounded-full bg-white/5 hover:bg-amber-500 hover:text-black text-white font-medium transition-all text-sm border border-white/10"
                >
                  <ShoppingCart className="w-4 h-4 text-amber-500 group-hover:text-black" />
                  {added === asset.id ? "Added" : "Add to Cart"} <ArrowRight className="w-4 h-4" />
                </button>
              </div>
            </div>
          </motion.div>
        ))}
      </div>

      {filtered.length === 0 && (
        <div className="rounded-3xl border border-white/8 bg-[#0A0A0E] p-10 text-center text-slate-400">
          No assets matched your current search.
        </div>
      )}
    </div>
  );
}

export default function Marketplace() {
  const [filter, setFilter] = useState("All");
  const [query, setQuery] = useState("");
  const [liveAssets, setLiveAssets] = useState<AssetCatalogItem[] | null>(null);

  useEffect(() => {
    let active = true;
    fetchAssets()
      .then((assets) => {
        if (!active) return;
        setLiveAssets(assets);
      })
      .catch(() => {
        // Keep null so the local fallback grid renders.
      });
    return () => {
      active = false;
    };
  }, []);

  // Prefer server-authoritative catalog; fall back to local data on failure/empty.
  if (liveAssets && liveAssets.length > 0) {
    return <LiveMarketplaceGrid assets={liveAssets} />;
  }

  const filteredAssets = ASSET_DATA.filter((item) => {
    const matchesCategory = filter === "All" || item.assetType === filter;
    const haystack = `${item.name} ${item.manufacturer} ${item.category} ${item.location} ${item.description}`.toLowerCase();
    return matchesCategory && haystack.includes(query.trim().toLowerCase());
  });

  return (
    <div className="max-w-7xl mx-auto px-6 py-8">
      <div className="flex flex-col md:flex-row justify-between items-start md:items-end mb-12 gap-6">
        <div>
          <h1 className="text-4xl md:text-5xl font-bold text-white mb-4">Luxury Marketplace</h1>
          <p className="text-lg text-slate-400 max-w-2xl">Browse and acquire the world's finest aircraft, real estate, and vehicles. Secure transactions via smart contracts and anonymous escrow.</p>
        </div>
        <div className="flex items-center gap-4 w-full md:w-auto">
          <div className="relative flex-1 md:w-64">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
            <input 
              type="text" 
              placeholder="Search assets..." 
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              className="w-full pl-11 pr-4 py-3 rounded-full bg-white/5 border border-white/10 text-white placeholder-slate-500 focus:outline-none focus:border-amber-500 transition-colors"
            />
          </div>
          <button className="p-3 rounded-full bg-white/5 border border-white/10 hover:bg-white/10 transition-colors">
            <Filter className="w-5 h-5 text-slate-300" />
          </button>
        </div>
      </div>

      {/* Categories */}
      <div className="flex overflow-x-auto pb-4 mb-8 gap-3 hide-scrollbar">
        {[
          { name: "All", icon: null },
          { name: "Aircraft", icon: Plane },
          { name: "Real Estate", icon: Home },
          { name: "Vehicles", icon: Car }
        ].map(cat => (
          <button 
            key={cat.name}
            onClick={() => setFilter(cat.name)}
            className={`whitespace-nowrap px-6 py-2 rounded-full text-sm font-medium transition-all flex items-center gap-2 ${filter === cat.name ? 'bg-amber-500 text-black shadow-[0_0_15px_rgba(245,158,11,0.3)]' : 'bg-white/5 text-slate-300 hover:bg-white/10 border border-white/10'}`}
          >
            {cat.icon && <cat.icon className={`w-4 h-4 ${filter === cat.name ? 'text-black' : 'text-slate-400'}`} />}
            {cat.name}
          </button>
        ))}
      </div>

      {/* Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 gap-8">
        {filteredAssets.map((asset, idx) => (
          <motion.div 
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: idx * 0.1 }}
            key={asset.id}
            className="group flex flex-col rounded-3xl bg-[#0A0A0E] border border-white/5 overflow-hidden hover:border-amber-500/30 transition-all duration-500"
          >
            <div className="relative h-72 overflow-hidden">
              <div 
                className="absolute inset-0 bg-cover bg-center transition-transform duration-700 group-hover:scale-105"
                style={{ backgroundImage: `url('${asset.image || FALLBACK_IMAGES[asset.assetType as keyof typeof FALLBACK_IMAGES]}')` }}
              />
              <div className="absolute inset-0 bg-gradient-to-t from-[#0A0A0E] via-transparent to-transparent" />
              
              {asset.featured && (
                <div className="absolute top-4 left-4 px-3 py-1 rounded-full bg-black/50 backdrop-blur-md border border-amber-500/30 flex items-center gap-1">
                  <Star className="w-3 h-3 text-amber-500 fill-amber-500" />
                  <span className="text-xs font-bold tracking-wider text-amber-500 uppercase">Featured</span>
                </div>
              )}
              
              <div className="absolute top-4 right-4 flex gap-2">
                <div className="px-3 py-1 rounded-full bg-black/50 backdrop-blur-md border border-white/10 flex items-center gap-1.5">
                  {asset.assetType === 'Aircraft' && <Plane className="w-3 h-3 text-slate-300" />}
                  {asset.assetType === 'Real Estate' && <Home className="w-3 h-3 text-slate-300" />}
                  {asset.assetType === 'Vehicles' && <Car className="w-3 h-3 text-slate-300" />}
                  <span className="text-xs font-bold tracking-wider text-slate-200 uppercase">{asset.assetType}</span>
                </div>
              </div>
            </div>

            <div className="p-6 md:p-8 flex-1 flex flex-col">
              <div className="flex justify-between items-start mb-4">
                <div>
                  <span className="text-xs font-medium text-amber-500 uppercase tracking-wider mb-1 block">{asset.category}</span>
                  <h3 className="text-2xl font-bold text-white">{asset.name}</h3>
                  <p className="text-sm text-slate-400">{asset.manufacturer}</p>
                </div>
                <div className="text-right">
                  <p className="text-xl font-bold text-white">{asset.price}</p>
                  <p className="text-sm text-slate-500">{asset.usdPrice}</p>
                </div>
              </div>

              {/* Dynamic Stats Row */}
              <div className="grid grid-cols-3 gap-4 py-6 my-auto border-t border-b border-white/5">
                {asset.assetType === 'Aircraft' && (
                  <>
                    <div className="flex flex-col gap-1">
                      <div className="flex items-center gap-1 text-slate-500">
                        <Compass className="w-4 h-4" />
                        <span className="text-xs uppercase tracking-wider">Range</span>
                      </div>
                      <span className="text-sm font-semibold text-slate-200">{asset.range}</span>
                    </div>
                    <div className="flex flex-col gap-1">
                      <div className="flex items-center gap-1 text-slate-500">
                        <Users className="w-4 h-4" />
                        <span className="text-xs uppercase tracking-wider">Pax</span>
                      </div>
                      <span className="text-sm font-semibold text-slate-200">{asset.passengers}</span>
                    </div>
                    <div className="flex flex-col gap-1">
                      <div className="flex items-center gap-1 text-slate-500">
                        <Fuel className="w-4 h-4" />
                        <span className="text-xs uppercase tracking-wider">Speed</span>
                      </div>
                      <span className="text-sm font-semibold text-slate-200">{asset.speed}</span>
                    </div>
                  </>
                )}
                
                {asset.assetType === 'Real Estate' && (
                  <>
                    <div className="flex flex-col gap-1">
                      <div className="flex items-center gap-1 text-slate-500">
                        <Maximize className="w-4 h-4" />
                        <span className="text-xs uppercase tracking-wider">Area</span>
                      </div>
                      <span className="text-sm font-semibold text-slate-200">{asset.area}</span>
                    </div>
                    <div className="flex flex-col gap-1">
                      <div className="flex items-center gap-1 text-slate-500">
                        <BedDouble className="w-4 h-4" />
                        <span className="text-xs uppercase tracking-wider">Beds</span>
                      </div>
                      <span className="text-sm font-semibold text-slate-200">{asset.beds}</span>
                    </div>
                    <div className="flex flex-col gap-1">
                      <div className="flex items-center gap-1 text-slate-500">
                        <Bath className="w-4 h-4" />
                        <span className="text-xs uppercase tracking-wider">Baths</span>
                      </div>
                      <span className="text-sm font-semibold text-slate-200">{asset.baths}</span>
                    </div>
                  </>
                )}

                {asset.assetType === 'Vehicles' && (
                  <>
                    <div className="flex flex-col gap-1">
                      <div className="flex items-center gap-1 text-slate-500">
                        <Zap className="w-4 h-4" />
                        <span className="text-xs uppercase tracking-wider">0-60</span>
                      </div>
                      <span className="text-sm font-semibold text-slate-200">{asset.zeroToSixty}</span>
                    </div>
                    <div className="flex flex-col gap-1">
                      <div className="flex items-center gap-1 text-slate-500">
                        <Gauge className="w-4 h-4" />
                        <span className="text-xs uppercase tracking-wider">Top Speed</span>
                      </div>
                      <span className="text-sm font-semibold text-slate-200">{asset.topSpeed}</span>
                    </div>
                    <div className="flex flex-col gap-1">
                      <div className="flex items-center gap-1 text-slate-500">
                        <Fuel className="w-4 h-4" />
                        <span className="text-xs uppercase tracking-wider">Power</span>
                      </div>
                      <span className="text-sm font-semibold text-slate-200">{asset.horsepower} hp</span>
                    </div>
                  </>
                )}
              </div>

              <div className="mt-6 flex items-center justify-between">
                <span className="text-sm text-slate-400 flex items-center gap-1">
                  <span className="w-2 h-2 rounded-full bg-emerald-500" />
                  {asset.location}
                </span>
                <Link 
                  to={`/marketplace/${asset.id}`} 
                  className="flex items-center gap-2 px-5 py-2.5 rounded-full bg-white/5 hover:bg-amber-500 hover:text-black text-white font-medium transition-all text-sm group-hover:border-amber-500/50 border border-white/10"
                >
                  View Details <ArrowRight className="w-4 h-4" />
                </Link>
              </div>
            </div>
          </motion.div>
        ))}
      </div>

      {filteredAssets.length === 0 && (
        <div className="rounded-3xl border border-white/8 bg-[#0A0A0E] p-10 text-center text-slate-400">
          No assets matched your current search.
        </div>
      )}
    </div>
  );
}
