import { motion } from "motion/react";
import { Link, useNavigate } from "react-router";
import { AlertCircle, ArrowRight, Boxes, Loader2, Search, ShoppingCart } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { fetchAssets, type AssetCatalogItem } from "../lib/backend";
import { addBackendAssetToCart } from "../lib/commerce";
import { assetImage, formatUsd } from "../lib/assetPresentation";

type LoadState =
  | { phase: "loading" }
  | { phase: "ready"; assets: AssetCatalogItem[] }
  | { phase: "error"; message: string };

/**
 * Marketplace grid, backed by GET /api/assets.
 *
 * This page used to fall back to a hardcoded ASSET_DATA grid whenever the
 * request failed or came back empty. That fallback showed priced inventory the
 * backend had never heard of: its ids matched no row, so nothing added from it
 * could become a booking. An empty catalogue and an unreachable backend are now
 * both reported as what they are, because inventing stock to fill the page is
 * worse than an honest empty state.
 */
export default function Marketplace() {
  const [state, setState] = useState<LoadState>({ phase: "loading" });

  useEffect(() => {
    let active = true;

    fetchAssets()
      .then((assets) => {
        if (!active) return;
        setState({ phase: "ready", assets });
      })
      .catch((error: unknown) => {
        if (!active) return;
        setState({
          phase: "error",
          message:
            error instanceof Error ? error.message : "Could not reach the asset catalogue.",
        });
      });

    return () => {
      active = false;
    };
  }, []);

  if (state.phase === "loading") {
    return (
      <Shell>
        <div className="flex flex-col items-center justify-center gap-4 py-24">
          <Loader2 className="h-6 w-6 animate-spin text-amber-500" />
          <p className="text-slate-400">Loading the live catalogue...</p>
        </div>
      </Shell>
    );
  }

  if (state.phase === "error") {
    return (
      <Shell>
        <div className="rounded-3xl border border-red-500/20 bg-red-500/5 p-10 text-center">
          <AlertCircle className="mx-auto mb-4 h-6 w-6 text-red-400" />
          <h2 className="mb-2 text-xl font-bold text-white">Catalogue unavailable</h2>
          <p className="text-slate-400">{state.message}</p>
        </div>
      </Shell>
    );
  }

  return <LiveMarketplaceGrid assets={state.assets} />;
}

function LiveMarketplaceGrid({ assets }: { assets: AssetCatalogItem[] }) {
  const navigate = useNavigate();
  const [filter, setFilter] = useState("All");
  const [query, setQuery] = useState("");
  const [added, setAdded] = useState<string | null>(null);

  // Categories come from the catalogue itself, so a new asset type appears
  // without a frontend change.
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
    <Shell>
      <div className="flex flex-col md:flex-row justify-between items-start md:items-end mb-12 gap-6">
        <div>
          <h1 className="text-4xl md:text-5xl font-bold text-white mb-4">Luxury Marketplace</h1>
          <p className="text-lg text-slate-400 max-w-2xl">
            Acquire tokenized shares in the world's finest aircraft, real estate, and vehicles.
            Prices and availability are pulled live from the AeroLedger backend.
          </p>
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
            <Link to={`/marketplace/${asset.id}`} className="relative h-64 overflow-hidden block">
              <div
                className="absolute inset-0 bg-cover bg-center transition-transform duration-700 group-hover:scale-105"
                style={{ backgroundImage: `url('${assetImage(asset)}')` }}
              />
              <div className="absolute inset-0 bg-gradient-to-t from-[#0A0A0E] via-transparent to-transparent" />
              <div className="absolute top-4 right-4 px-3 py-1 rounded-full bg-black/50 backdrop-blur-md border border-white/10 flex items-center gap-1.5">
                <Boxes className="w-3 h-3 text-slate-300" />
                <span className="text-xs font-bold tracking-wider text-slate-200 uppercase">
                  {asset.type}
                </span>
              </div>
            </Link>

            <div className="p-6 md:p-8 flex-1 flex flex-col">
              <div className="flex justify-between items-start mb-4">
                <div>
                  <span className="text-xs font-medium text-amber-500 uppercase tracking-wider mb-1 block">
                    {asset.type}
                  </span>
                  <h3 className="text-2xl font-bold text-white">{asset.name}</h3>
                </div>
                <div className="text-right">
                  <p className="text-xl font-bold text-white">{formatUsd(asset.token_price)}</p>
                  <p className="text-sm text-slate-500">per token</p>
                </div>
              </div>

              {asset.description && (
                <p className="text-sm text-slate-400 mb-6">{asset.description}</p>
              )}

              <div className="grid grid-cols-2 gap-4 py-6 mt-auto border-t border-b border-white/5">
                <div className="flex flex-col gap-1">
                  <span className="text-xs uppercase tracking-wider text-slate-500">
                    Total value
                  </span>
                  <span className="text-sm font-semibold text-slate-200">
                    {formatUsd(asset.total_value)}
                  </span>
                </div>
                <div className="flex flex-col gap-1">
                  <span className="text-xs uppercase tracking-wider text-slate-500">
                    Available tokens
                  </span>
                  <span className="text-sm font-semibold text-slate-200">
                    {asset.available_tokens.toLocaleString()} /{" "}
                    {asset.total_tokens.toLocaleString()}
                  </span>
                </div>
              </div>

              <div className="mt-6 flex items-center justify-between gap-4">
                <Link
                  to={`/marketplace/${asset.id}`}
                  className="text-sm font-medium text-slate-400 hover:text-amber-500 transition-colors"
                >
                  View details
                </Link>
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
          {assets.length === 0
            ? "The catalogue is empty. No assets are currently listed."
            : "No assets matched your current search."}
        </div>
      )}
    </Shell>
  );
}

function Shell({ children }: { children: React.ReactNode }) {
  return <div className="max-w-7xl mx-auto px-6 py-8">{children}</div>;
}
