import { useParams, Link, useNavigate } from "react-router";
import { motion } from "motion/react";
import { ASSET_DATA } from "../data/assets";
import { ArrowLeft, Bitcoin, Compass, Users, Fuel, Maximize, BedDouble, Bath, Zap, Gauge, MapPin, Calendar, ExternalLink, ShoppingCart } from "lucide-react";
import { useState } from "react";

import { addAssetToCart } from "../lib/commerce";

export default function AssetDetails() {
  const { id } = useParams();
  const navigate = useNavigate();
  const asset = ASSET_DATA.find((a) => a.id === id);
  const [added, setAdded] = useState(false);

  if (!asset) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh]">
        <h2 className="text-2xl font-bold text-white mb-4">Asset Not Found</h2>
        <Link to="/marketplace" className="text-amber-500 hover:underline flex items-center gap-2">
          <ArrowLeft className="w-4 h-4" /> Back to Marketplace
        </Link>
      </div>
    );
  }

  const handleAddToCart = () => {
    addAssetToCart(asset.id);
    setAdded(true);
    window.setTimeout(() => setAdded(false), 1800);
  };

  return (
    <div className="max-w-7xl mx-auto px-6 py-8">
      <Link to="/marketplace" className="inline-flex items-center gap-2 text-slate-400 hover:text-white transition-colors mb-8 text-sm font-medium">
        <ArrowLeft className="w-4 h-4" /> Back to Marketplace
      </Link>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-12">
        <div>
          <motion.div initial={{ opacity: 0, scale: 0.95 }} animate={{ opacity: 1, scale: 1 }} className="rounded-3xl overflow-hidden aspect-square relative shadow-2xl shadow-black/50">
            <div className="absolute inset-0 bg-cover bg-center" style={{ backgroundImage: `url('${asset.image}')` }} />
            <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent" />
            <div className="absolute bottom-6 left-6 right-6 flex justify-between items-end">
              <div className="px-4 py-2 bg-black/50 backdrop-blur-md rounded-full border border-white/10 inline-flex items-center gap-2">
                <MapPin className="w-4 h-4 text-amber-500" />
                <span className="text-sm font-medium text-white">{asset.location}</span>
              </div>
            </div>
          </motion.div>
        </div>

        <div className="flex flex-col">
          <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.1 }}>
            <div className="flex items-center gap-3 mb-4">
              <span className="px-3 py-1 rounded-full bg-amber-500/10 text-amber-500 border border-amber-500/20 text-xs font-bold tracking-wider uppercase">{asset.assetType}</span>
              <span className="text-slate-400 text-sm">•</span>
              <span className="text-slate-400 text-sm uppercase tracking-wider font-semibold">{asset.category}</span>
            </div>

            <h1 className="text-4xl md:text-5xl font-bold text-white mb-2">{asset.name}</h1>
            <p className="text-xl text-slate-400 mb-8">{asset.manufacturer}</p>

            <div className="p-6 rounded-2xl bg-white/5 border border-white/10 mb-8 flex flex-col gap-5">
              <div className="flex flex-col sm:flex-row justify-between items-center sm:items-start gap-4">
                <div>
                  <p className="text-sm text-slate-400 mb-1">Asking Price</p>
                  <p className="text-3xl font-bold text-white flex items-center gap-2">
                    <Bitcoin className="w-8 h-8 text-amber-500" /> {asset.price}
                  </p>
                  <p className="text-sm text-slate-500 mt-1">Approx. {asset.usdPrice}</p>
                </div>
                {added && <span className="rounded-full border border-emerald-500/20 bg-emerald-500/10 px-4 py-2 text-sm text-emerald-400">Added to cart</span>}
              </div>

              <div className="flex flex-col sm:flex-row gap-3">
                <button type="button" onClick={handleAddToCart} className="w-full sm:w-auto px-6 py-4 border border-white/10 text-white font-semibold rounded-xl hover:bg-white/5 transition-all inline-flex items-center justify-center gap-2">
                  <ShoppingCart className="h-4 w-4 text-amber-500" /> Add to Cart
                </button>
                <button type="button" onClick={() => { addAssetToCart(asset.id); navigate(`/cart?added=${asset.id}`); }} className="w-full sm:w-auto px-6 py-4 bg-amber-500 hover:bg-amber-400 text-black font-bold rounded-xl transition-all shadow-[0_0_20px_rgba(245,158,11,0.3)] hover:shadow-[0_0_30px_rgba(245,158,11,0.5)] text-center whitespace-nowrap">
                  Review in Cart
                </button>
              </div>
            </div>

            <div className="mb-8">
              <h3 className="text-lg font-bold text-white mb-4">Overview</h3>
              <p className="text-slate-400 leading-relaxed">{asset.description || "No description available for this luxury asset. Contact our concierge for detailed specifications and provenance reports."}</p>
            </div>

            <h3 className="text-lg font-bold text-white mb-4">Specifications</h3>
            <div className="grid grid-cols-2 sm:grid-cols-3 gap-4">
              <div className="p-4 rounded-xl bg-[#0A0A0E] border border-white/5 flex flex-col gap-2">
                <Calendar className="w-5 h-5 text-slate-500" />
                <span className="text-xs text-slate-500 uppercase tracking-wider">Year</span>
                <span className="text-sm font-semibold text-white">{asset.year}</span>
              </div>

              {asset.assetType === "Aircraft" && (
                <>
                  <div className="p-4 rounded-xl bg-[#0A0A0E] border border-white/5 flex flex-col gap-2"><Compass className="w-5 h-5 text-slate-500" /><span className="text-xs text-slate-500 uppercase tracking-wider">Range</span><span className="text-sm font-semibold text-white">{asset.range}</span></div>
                  <div className="p-4 rounded-xl bg-[#0A0A0E] border border-white/5 flex flex-col gap-2"><Users className="w-5 h-5 text-slate-500" /><span className="text-xs text-slate-500 uppercase tracking-wider">Passengers</span><span className="text-sm font-semibold text-white">{asset.passengers}</span></div>
                  <div className="p-4 rounded-xl bg-[#0A0A0E] border border-white/5 flex flex-col gap-2"><Fuel className="w-5 h-5 text-slate-500" /><span className="text-xs text-slate-500 uppercase tracking-wider">Speed</span><span className="text-sm font-semibold text-white">{asset.speed}</span></div>
                </>
              )}

              {asset.assetType === "Real Estate" && (
                <>
                  <div className="p-4 rounded-xl bg-[#0A0A0E] border border-white/5 flex flex-col gap-2"><Maximize className="w-5 h-5 text-slate-500" /><span className="text-xs text-slate-500 uppercase tracking-wider">Area</span><span className="text-sm font-semibold text-white">{asset.area}</span></div>
                  <div className="p-4 rounded-xl bg-[#0A0A0E] border border-white/5 flex flex-col gap-2"><BedDouble className="w-5 h-5 text-slate-500" /><span className="text-xs text-slate-500 uppercase tracking-wider">Bedrooms</span><span className="text-sm font-semibold text-white">{asset.beds}</span></div>
                  <div className="p-4 rounded-xl bg-[#0A0A0E] border border-white/5 flex flex-col gap-2"><Bath className="w-5 h-5 text-slate-500" /><span className="text-xs text-slate-500 uppercase tracking-wider">Bathrooms</span><span className="text-sm font-semibold text-white">{asset.baths}</span></div>
                </>
              )}

              {asset.assetType === "Vehicles" && (
                <>
                  <div className="p-4 rounded-xl bg-[#0A0A0E] border border-white/5 flex flex-col gap-2"><Zap className="w-5 h-5 text-slate-500" /><span className="text-xs text-slate-500 uppercase tracking-wider">0-60 mph</span><span className="text-sm font-semibold text-white">{asset.zeroToSixty}</span></div>
                  <div className="p-4 rounded-xl bg-[#0A0A0E] border border-white/5 flex flex-col gap-2"><Gauge className="w-5 h-5 text-slate-500" /><span className="text-xs text-slate-500 uppercase tracking-wider">Top Speed</span><span className="text-sm font-semibold text-white">{asset.topSpeed}</span></div>
                  <div className="p-4 rounded-xl bg-[#0A0A0E] border border-white/5 flex flex-col gap-2"><Fuel className="w-5 h-5 text-slate-500" /><span className="text-xs text-slate-500 uppercase tracking-wider">Horsepower</span><span className="text-sm font-semibold text-white">{asset.horsepower} hp</span></div>
                </>
              )}
            </div>

            <div className="mt-8 pt-8 border-t border-white/10">
              <button className="flex items-center gap-2 text-slate-400 hover:text-amber-500 transition-colors font-medium">
                <ExternalLink className="w-4 h-4" /> Download Due Diligence Report
              </button>
            </div>
          </motion.div>
        </div>
      </div>
    </div>
  );
}
