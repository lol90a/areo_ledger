import { useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router";
import { motion } from "motion/react";
import {
  AlertCircle,
  ArrowLeft,
  BadgeCheck,
  Boxes,
  Coins,
  Layers,
  Loader2,
  ShoppingCart,
  TrendingUp,
} from "lucide-react";

import { fetchAssets, type AssetCatalogItem } from "../lib/backend";
import { addBackendAssetToCart } from "../lib/commerce";
import { assetImage, formatUsd } from "../lib/assetPresentation";

type LoadState =
  | { phase: "loading" }
  | { phase: "ready"; asset: AssetCatalogItem }
  | { phase: "missing" }
  | { phase: "error"; message: string };

/**
 * Detail view for one marketplace asset.
 *
 * Everything on this page comes from GET /api/assets. That matters for more than
 * freshness: the cart item has to carry the backend's own asset id, because
 * checkout sends it as `asset_id` and the server prices the booking from that
 * row. This page previously rendered a hardcoded catalogue whose ids ("ac-001")
 * no backend row shared, so anything added from here reached checkout with no
 * asset id at all and could never become a booking.
 *
 * There is no GET /api/assets/:id endpoint, so the asset is resolved from the
 * list. The catalogue is small and already fetched on the marketplace page, so
 * one extra request costs less than widening the backend's public surface.
 */
export default function AssetDetails() {
  const { id } = useParams();
  const navigate = useNavigate();
  const [state, setState] = useState<LoadState>({ phase: "loading" });
  const [added, setAdded] = useState(false);

  useEffect(() => {
    let active = true;

    fetchAssets()
      .then((assets) => {
        if (!active) return;
        const match = assets.find((asset) => asset.id === id);
        setState(match ? { phase: "ready", asset: match } : { phase: "missing" });
      })
      .catch((error: unknown) => {
        if (!active) return;
        setState({
          phase: "error",
          message: error instanceof Error ? error.message : "Could not reach the catalogue.",
        });
      });

    return () => {
      active = false;
    };
  }, [id]);

  if (state.phase === "loading") {
    return (
      <StatusPanel>
        <Loader2 className="h-6 w-6 animate-spin text-amber-500" />
        <p className="text-slate-400">Loading asset from the catalogue...</p>
      </StatusPanel>
    );
  }

  if (state.phase === "error") {
    return (
      <StatusPanel>
        <AlertCircle className="h-6 w-6 text-red-400" />
        <h2 className="text-2xl font-bold text-white">Catalogue unavailable</h2>
        <p className="max-w-md text-center text-slate-400">{state.message}</p>
        <BackToMarketplace />
      </StatusPanel>
    );
  }

  if (state.phase === "missing") {
    return (
      <StatusPanel>
        <h2 className="text-2xl font-bold text-white">Asset not found</h2>
        <p className="max-w-md text-center text-slate-400">
          This asset is no longer listed in the marketplace.
        </p>
        <BackToMarketplace />
      </StatusPanel>
    );
  }

  const { asset } = state;
  // The list endpoint only returns active rows, so an asset that reaches this
  // page is sellable unless every token is already spoken for.
  const soldOut = asset.available_tokens <= 0;

  const addToCart = () => {
    addBackendAssetToCart(asset);
    setAdded(true);
    window.setTimeout(() => setAdded(false), 1800);
  };

  return (
    <div className="max-w-7xl mx-auto px-6 py-8">
      <Link
        to="/marketplace"
        className="inline-flex items-center gap-2 text-slate-400 hover:text-white transition-colors mb-8 text-sm font-medium"
      >
        <ArrowLeft className="w-4 h-4" /> Back to Marketplace
      </Link>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-12">
        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          className="rounded-3xl overflow-hidden aspect-square relative shadow-2xl shadow-black/50"
        >
          <div
            className="absolute inset-0 bg-cover bg-center"
            style={{ backgroundImage: `url('${assetImage(asset)}')` }}
          />
          <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent" />
          <div className="absolute bottom-6 left-6 px-4 py-2 bg-black/50 backdrop-blur-md rounded-full border border-white/10 inline-flex items-center gap-2">
            <Boxes className="w-4 h-4 text-amber-500" />
            <span className="text-sm font-medium text-white capitalize">{asset.type}</span>
          </div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="flex flex-col"
        >
          <div className="flex items-center gap-3 mb-4">
            <span className="px-3 py-1 rounded-full bg-amber-500/10 text-amber-500 border border-amber-500/20 text-xs font-bold tracking-wider uppercase">
              {asset.type}
            </span>
            {asset.status === "active" && (
              <span className="inline-flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-emerald-400">
                <BadgeCheck className="h-3.5 w-3.5" /> Listed
              </span>
            )}
          </div>

          <h1 className="text-4xl md:text-5xl font-bold text-white mb-8">{asset.name}</h1>

          <div className="p-6 rounded-2xl bg-white/5 border border-white/10 mb-8 flex flex-col gap-5">
            <div className="flex flex-col sm:flex-row justify-between items-center sm:items-start gap-4">
              <div>
                <p className="text-sm text-slate-400 mb-1">Token price</p>
                <p className="text-3xl font-bold text-white flex items-center gap-2">
                  <Coins className="w-7 h-7 text-amber-500" /> {formatUsd(asset.token_price)}
                </p>
                <p className="text-sm text-slate-500 mt-1">
                  Settled in crypto at checkout at the live rate.
                </p>
              </div>
              {added && (
                <span className="rounded-full border border-emerald-500/20 bg-emerald-500/10 px-4 py-2 text-sm text-emerald-400">
                  Added to cart
                </span>
              )}
            </div>

            {soldOut ? (
              <p className="rounded-xl border border-white/10 bg-black/30 px-4 py-3 text-sm text-slate-400">
                Every token in this asset is currently allocated.
              </p>
            ) : (
              <div className="flex flex-col sm:flex-row gap-3">
                <button
                  type="button"
                  onClick={addToCart}
                  className="w-full sm:w-auto px-6 py-4 border border-white/10 text-white font-semibold rounded-xl hover:bg-white/5 transition-all inline-flex items-center justify-center gap-2"
                >
                  <ShoppingCart className="h-4 w-4 text-amber-500" /> Add to Cart
                </button>
                <button
                  type="button"
                  onClick={() => {
                    addBackendAssetToCart(asset);
                    navigate("/cart?added=asset");
                  }}
                  className="w-full sm:w-auto px-6 py-4 bg-amber-500 hover:bg-amber-400 text-black font-bold rounded-xl transition-all shadow-[0_0_20px_rgba(245,158,11,0.3)] hover:shadow-[0_0_30px_rgba(245,158,11,0.5)] text-center whitespace-nowrap"
                >
                  Review in Cart
                </button>
              </div>
            )}
          </div>

          {asset.description && (
            <div className="mb-8">
              <h3 className="text-lg font-bold text-white mb-4">Overview</h3>
              <p className="text-slate-400 leading-relaxed">{asset.description}</p>
            </div>
          )}

          <h3 className="text-lg font-bold text-white mb-4">Holding</h3>
          <div className="grid grid-cols-2 sm:grid-cols-3 gap-4">
            <Spec icon={<TrendingUp className="w-5 h-5 text-slate-500" />} label="Total value">
              {formatUsd(asset.total_value)}
            </Spec>
            <Spec icon={<Coins className="w-5 h-5 text-slate-500" />} label="Token price">
              {formatUsd(asset.token_price)}
            </Spec>
            <Spec icon={<Layers className="w-5 h-5 text-slate-500" />} label="Total tokens">
              {asset.total_tokens.toLocaleString()}
            </Spec>
            <Spec icon={<Boxes className="w-5 h-5 text-slate-500" />} label="Available">
              {asset.available_tokens.toLocaleString()}
            </Spec>
          </div>
        </motion.div>
      </div>
    </div>
  );
}

function Spec({
  icon,
  label,
  children,
}: {
  icon: React.ReactNode;
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="p-4 rounded-xl bg-[#0A0A0E] border border-white/5 flex flex-col gap-2">
      {icon}
      <span className="text-xs text-slate-500 uppercase tracking-wider">{label}</span>
      <span className="text-sm font-semibold text-white">{children}</span>
    </div>
  );
}

function StatusPanel({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex flex-col items-center justify-center gap-4 min-h-[60vh]">{children}</div>
  );
}

function BackToMarketplace() {
  return (
    <Link to="/marketplace" className="text-amber-500 hover:underline flex items-center gap-2">
      <ArrowLeft className="w-4 h-4" /> Back to Marketplace
    </Link>
  );
}
