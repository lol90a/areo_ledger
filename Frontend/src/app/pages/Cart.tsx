import { useEffect, useMemo, useState } from "react";
import { Link, useSearchParams } from "react-router";
import { motion } from "motion/react";
import { ArrowRight, BriefcaseBusiness, Plane, ShoppingCart, Trash2, Wallet } from "lucide-react";

import { cartTotalSummary, getCartItems, removeCartItem, type CartItem } from "../lib/commerce";

function itemIcon(item: CartItem) {
  return item.kind === "flight" ? Plane : BriefcaseBusiness;
}

export default function Cart() {
  const [searchParams] = useSearchParams();
  const added = searchParams.get("added");
  const [items, setItems] = useState<CartItem[]>([]);

  useEffect(() => {
    const sync = () => setItems(getCartItems());
    sync();
    window.addEventListener("aeroledger-cart-updated", sync);
    return () => window.removeEventListener("aeroledger-cart-updated", sync);
  }, []);

  const summary = useMemo(() => cartTotalSummary(items), [items]);

  return (
    <div className="max-w-7xl mx-auto px-6 py-8">
      <div className="flex flex-col lg:flex-row justify-between items-start gap-8 mb-10">
        <div>
          <h1 className="text-4xl md:text-5xl font-bold text-white mb-3">Client Cart</h1>
          <p className="text-slate-400 max-w-2xl">
            Review flight reservations and acquisition interests before moving each item into blockchain settlement.
          </p>
          {added && (
            <p className="mt-4 inline-flex rounded-full border border-emerald-500/20 bg-emerald-500/10 px-4 py-2 text-sm text-emerald-400">
              Item added to cart and ready for checkout.
            </p>
          )}
        </div>

        <div className="w-full lg:w-80 rounded-3xl border border-white/10 bg-[#0A0A0E] p-6">
          <p className="text-xs uppercase tracking-[0.24em] text-slate-500 mb-2">Settlement Snapshot</p>
          <p className="text-3xl font-bold text-white">{summary.itemCount} Items</p>
          <p className="mt-2 text-sm text-slate-400">Each cart item is settled individually to preserve payment verification and audit history.</p>
          <div className="mt-6 border-t border-white/10 pt-6">
            <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Preview</p>
            <p className="mt-2 text-sm text-white">{summary.preview || "Cart is currently empty"}</p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-[1fr_340px] gap-8">
        <div className="space-y-5">
          {items.length === 0 ? (
            <div className="rounded-3xl border border-white/8 bg-[#0A0A0E] p-10 text-center shadow-2xl shadow-black/30">
              <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-white/[0.04] border border-white/10">
                <ShoppingCart className="h-7 w-7 text-amber-500" />
              </div>
              <h2 className="mt-6 text-2xl font-bold text-white">Your cart is empty</h2>
              <p className="mt-3 text-slate-400 max-w-xl mx-auto">
                Add a flight or acquisition from the marketplace to begin the checkout and payment verification flow.
              </p>
              <div className="mt-8 flex flex-col sm:flex-row justify-center gap-3">
                <Link to="/flights" className="inline-flex items-center justify-center gap-2 rounded-2xl bg-amber-500 px-6 py-4 font-semibold text-black transition-all hover:bg-amber-400">
                  Browse Flights <ArrowRight className="h-4 w-4" />
                </Link>
                <Link to="/marketplace" className="inline-flex items-center justify-center gap-2 rounded-2xl border border-white/10 px-6 py-4 font-semibold text-white transition-all hover:bg-white/[0.04]">
                  Browse Marketplace
                </Link>
              </div>
            </div>
          ) : (
            items.map((item, index) => {
              const Icon = itemIcon(item);
              return (
                <motion.div
                  key={item.id}
                  initial={{ opacity: 0, y: 16 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: index * 0.06 }}
                  className="overflow-hidden rounded-3xl border border-white/8 bg-[#0A0A0E] p-6 shadow-2xl shadow-black/30"
                >
                  <div className="flex flex-col lg:flex-row gap-6">
                    <div
                      className="h-52 lg:h-auto lg:w-72 rounded-2xl bg-cover bg-center border border-white/10"
                      style={{ backgroundImage: item.image ? `url('${item.image}')` : undefined }}
                    />

                    <div className="flex-1 flex flex-col justify-between gap-5">
                      <div>
                        <div className="flex items-center justify-between gap-4">
                          <div className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs uppercase tracking-[0.24em] text-slate-400">
                            <Icon className="h-3.5 w-3.5 text-amber-500" />
                            {item.category}
                          </div>
                          <button
                            type="button"
                            onClick={() => setItems(removeCartItem(item.id))}
                            className="inline-flex items-center gap-2 text-sm text-slate-400 transition-colors hover:text-rose-400"
                          >
                            <Trash2 className="h-4 w-4" /> Remove
                          </button>
                        </div>

                        <h2 className="mt-4 text-2xl font-bold text-white">{item.name}</h2>
                        <p className="mt-2 text-slate-400">{item.subtitle}</p>

                        <div className="mt-6 grid grid-cols-1 md:grid-cols-2 gap-4">
                          <div className="rounded-2xl border border-white/8 bg-white/[0.03] p-4">
                            <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Primary detail</p>
                            <p className="mt-2 text-sm font-medium text-white">{item.primaryMeta}</p>
                          </div>
                          <div className="rounded-2xl border border-white/8 bg-white/[0.03] p-4">
                            <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Secondary detail</p>
                            <p className="mt-2 text-sm font-medium text-white">{item.secondaryMeta}</p>
                          </div>
                        </div>
                      </div>

                      <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-5 border-t border-white/10 pt-5">
                        <div>
                          <p className="text-sm text-slate-500">Estimated settlement</p>
                          <p className="text-2xl font-bold text-white">{item.cryptoPrice}</p>
                          <p className="text-sm text-slate-500">{item.fiatPrice}</p>
                        </div>

                        <div className="flex flex-col sm:flex-row gap-3">
                          <Link
                            to={`/checkout?itemId=${item.id}`}
                            className="inline-flex items-center justify-center gap-2 rounded-2xl bg-amber-500 px-6 py-4 font-semibold text-black transition-all hover:bg-amber-400"
                          >
                            Proceed to Checkout <ArrowRight className="h-4 w-4" />
                          </Link>
                        </div>
                      </div>
                    </div>
                  </div>
                </motion.div>
              );
            })
          )}
        </div>

        <div className="rounded-3xl border border-white/8 bg-[#0A0A0E] p-6 h-fit">
          <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Crypto Checkout Flow</p>
          <div className="mt-5 space-y-4">
            {[
              "Review cart item and lock settlement path",
              "Submit traveler or buyer details",
              "Initialize payment wallet instructions",
              "Submit blockchain transaction hash",
              "Track initialized, awaiting, and confirmed states",
            ].map((step, index) => (
              <div key={step} className="flex gap-3">
                <div className="mt-1 flex h-6 w-6 items-center justify-center rounded-full border border-amber-500/30 bg-amber-500/10 text-xs font-semibold text-amber-400">
                  {index + 1}
                </div>
                <p className="text-sm leading-6 text-slate-300">{step}</p>
              </div>
            ))}
          </div>

          <div className="mt-8 rounded-2xl border border-white/8 bg-white/[0.03] p-5">
            <div className="flex items-center gap-3 text-slate-300">
              <Wallet className="h-5 w-5 text-amber-500" />
              <span className="text-sm font-medium">Treasury will verify each order independently</span>
            </div>
            <p className="mt-3 text-sm leading-6 text-slate-500">
              Orders are separated to keep audit trails, transaction hashes, and blockchain confirmations clear for every booking or acquisition.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
