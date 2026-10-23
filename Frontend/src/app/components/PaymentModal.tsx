/**
 * Pre-checkout confirmation for a marketplace asset.
 *
 * This component used to run `setTimeout(() => setStep("success"), 3000)` and show
 * "Payment Successful" with a green tick. Nothing had been priced, no wallet had
 * been touched, and no chain had been consulted: a timer was being presented to a
 * customer as financial settlement. That is removed entirely.
 *
 * What it does now is hand the user to the real checkout, where the backend issues
 * a signed, expiring quote, the wallet balance is checked, and "paid" is only ever
 * shown after the backend has verified the transfer on-chain. The prices shown here
 * are catalogue figures and are labelled as indicative, because the authoritative
 * amount is the one the server quotes at checkout.
 */
import { AnimatePresence, motion } from "motion/react";
import { ArrowRight, Info, ShieldCheck, Wallet, X } from "lucide-react";
import { useNavigate } from "react-router";

interface PaymentModalProps {
  isOpen: boolean;
  onClose: () => void;
  asset: {
    title: string;
    price: string;
    image: string;
  } | null;
}

export function PaymentModal({ isOpen, onClose, asset }: PaymentModalProps) {
  const navigate = useNavigate();

  if (!isOpen || !asset) return null;

  const goToCheckout = () => {
    onClose();
    navigate("/checkout");
  };

  return (
    <AnimatePresence>
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
          className="absolute inset-0 bg-[#05050A]/90 backdrop-blur-xl"
        />

        <motion.div
          initial={{ opacity: 0, scale: 0.95, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.95, y: 20 }}
          className="relative w-full max-w-lg overflow-hidden rounded-3xl border border-white/10 bg-[#0A0A10] shadow-[0_0_50px_rgba(0,0,0,0.5)]"
        >
          <div className="flex items-center justify-between border-b border-white/5 p-6">
            <h3 className="flex items-center gap-2 text-xl font-bold text-white">
              <Wallet className="h-5 w-5 text-amber-500" />
              Start Secure Checkout
            </h3>
            <button
              onClick={onClose}
              className="rounded-full p-2 text-slate-400 transition-colors hover:bg-white/5 hover:text-white"
            >
              <X className="h-5 w-5" />
            </button>
          </div>

          <div className="space-y-6 p-6">
            <div className="flex gap-4 rounded-2xl border border-white/10 bg-white/5 p-4">
              <div
                className="h-20 w-20 rounded-xl bg-cover bg-center"
                style={{ backgroundImage: `url(${asset.image})` }}
              />
              <div className="flex flex-1 flex-col justify-center">
                <p className="mb-1 text-sm font-medium text-slate-400">Selected</p>
                <h4 className="text-lg font-bold leading-tight text-white">{asset.title}</h4>
                <p className="mt-1 text-sm text-slate-500">
                  Listed at {asset.price}
                  <span className="ml-1 text-slate-600">(indicative)</span>
                </p>
              </div>
            </div>

            <div className="flex gap-3 rounded-xl border border-white/10 bg-white/[0.03] p-4">
              <Info className="h-5 w-5 shrink-0 text-slate-400" />
              <p className="text-sm leading-relaxed text-slate-400">
                The figure above is the catalogue price. The amount you actually pay is
                quoted by the backend at checkout, held for a fixed window, and shown to you
                in both {asset.price.replace(/[\d.,\s]/g, "") || "fiat"} and the exact token
                amount before you approve anything.
              </p>
            </div>

            <div className="flex gap-3 rounded-xl border border-amber-500/20 bg-amber-500/10 p-4">
              <ShieldCheck className="h-5 w-5 shrink-0 text-amber-500" />
              <p className="text-sm leading-relaxed text-amber-200/80">
                At checkout you will connect a wallet, see whether your balance covers the
                amount, and approve the price explicitly. An order is only marked paid once
                the backend has verified the transfer on-chain.
              </p>
            </div>

            <button
              onClick={goToCheckout}
              className="flex w-full items-center justify-center gap-2 rounded-xl bg-amber-500 py-4 font-bold uppercase tracking-wider text-black shadow-[0_0_20px_rgba(245,158,11,0.2)] transition-all hover:bg-amber-400 hover:shadow-[0_0_30px_rgba(245,158,11,0.4)]"
            >
              Continue to Checkout <ArrowRight className="h-5 w-5" />
            </button>
          </div>
        </motion.div>
      </div>
    </AnimatePresence>
  );
}
