import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "motion/react";
import { 
  X, 
  Wallet, 
  CheckCircle2, 
  Loader2, 
  ArrowRight,
  ShieldCheck
} from "lucide-react";

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
  const [step, setStep] = useState<"confirm" | "processing" | "success">("confirm");

  useEffect(() => {
    if (isOpen) setStep("confirm");
  }, [isOpen]);

  const handlePay = () => {
    setStep("processing");
    setTimeout(() => {
      setStep("success");
    }, 3000);
  };

  if (!isOpen || !asset) return null;

  return (
    <AnimatePresence>
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        {/* Backdrop */}
        <motion.div 
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={step === "processing" ? undefined : onClose}
          className="absolute inset-0 bg-[#05050A]/90 backdrop-blur-xl"
        />

        {/* Modal */}
        <motion.div 
          initial={{ opacity: 0, scale: 0.95, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.95, y: 20 }}
          className="relative w-full max-w-lg bg-[#0A0A10] border border-white/10 rounded-3xl shadow-[0_0_50px_rgba(0,0,0,0.5)] overflow-hidden"
        >
          {/* Header */}
          <div className="flex items-center justify-between p-6 border-b border-white/5">
            <h3 className="text-xl font-bold text-white flex items-center gap-2">
              <Wallet className="w-5 h-5 text-amber-500" />
              Secure Checkout
            </h3>
            {step !== "processing" && (
              <button 
                onClick={onClose}
                className="p-2 rounded-full hover:bg-white/5 text-slate-400 hover:text-white transition-colors"
              >
                <X className="w-5 h-5" />
              </button>
            )}
          </div>

          <div className="p-6">
            {step === "confirm" && (
              <motion.div
                initial={{ opacity: 0, x: 20 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -20 }}
                className="space-y-6"
              >
                {/* Asset Summary */}
                <div className="flex gap-4 p-4 rounded-2xl bg-white/5 border border-white/10">
                  <div 
                    className="w-20 h-20 rounded-xl bg-cover bg-center"
                    style={{ backgroundImage: `url(${asset.image})` }}
                  />
                  <div className="flex-1 flex flex-col justify-center">
                    <p className="text-slate-400 text-sm font-medium mb-1">Purchasing</p>
                    <h4 className="text-white font-bold text-lg leading-tight">{asset.title}</h4>
                  </div>
                </div>

                {/* Payment Details */}
                <div className="space-y-4">
                  <div className="flex justify-between items-center py-3 border-b border-white/5">
                    <span className="text-slate-400 font-medium">Network Fee</span>
                    <span className="text-white">0.001 BTC</span>
                  </div>
                  <div className="flex justify-between items-center py-3 border-b border-white/5">
                    <span className="text-slate-400 font-medium">Total Amount</span>
                    <span className="text-2xl font-bold text-amber-400">{asset.price}</span>
                  </div>
                </div>

                <div className="bg-amber-500/10 border border-amber-500/20 rounded-xl p-4 flex gap-3">
                  <ShieldCheck className="w-5 h-5 text-amber-500 shrink-0" />
                  <p className="text-sm text-amber-200/80 leading-relaxed">
                    Transaction will be processed securely via the Bitcoin network. Ensure you have sufficient balance.
                  </p>
                </div>

                <button 
                  onClick={handlePay}
                  className="w-full py-4 rounded-xl bg-amber-500 hover:bg-amber-400 text-black font-bold tracking-wider uppercase flex items-center justify-center gap-2 transition-all shadow-[0_0_20px_rgba(245,158,11,0.2)] hover:shadow-[0_0_30px_rgba(245,158,11,0.4)]"
                >
                  Confirm Payment <ArrowRight className="w-5 h-5" />
                </button>
              </motion.div>
            )}

            {step === "processing" && (
              <motion.div
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                className="py-12 flex flex-col items-center justify-center text-center"
              >
                <div className="relative mb-8">
                  <div className="w-24 h-24 border-4 border-amber-500/20 rounded-full" />
                  <div className="absolute inset-0 w-24 h-24 border-4 border-amber-500 border-t-transparent rounded-full animate-spin" />
                  <div className="absolute inset-0 flex items-center justify-center">
                    <Wallet className="w-8 h-8 text-amber-500 animate-pulse" />
                  </div>
                </div>
                <h4 className="text-xl font-bold text-white mb-2">Processing Transaction</h4>
                <p className="text-slate-400 max-w-[250px]">
                  Confirming on the blockchain. This usually takes a few moments...
                </p>
              </motion.div>
            )}

            {step === "success" && (
              <motion.div
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                className="py-12 flex flex-col items-center justify-center text-center"
              >
                <div className="w-24 h-24 rounded-full bg-emerald-500/10 flex items-center justify-center mb-6 shadow-[0_0_30px_rgba(16,185,129,0.2)] relative">
                  <motion.div 
                    initial={{ scale: 0 }}
                    animate={{ scale: 1 }}
                    transition={{ type: "spring", stiffness: 200, damping: 20 }}
                  >
                    <CheckCircle2 className="w-12 h-12 text-emerald-500" />
                  </motion.div>
                </div>
                <h4 className="text-2xl font-bold text-white mb-2">Payment Successful</h4>
                <p className="text-slate-400 max-w-[300px] mb-8">
                  Your purchase of <strong className="text-white">{asset.title}</strong> has been confirmed.
                </p>
                <button 
                  onClick={onClose}
                  className="w-full py-4 rounded-xl bg-white/5 hover:bg-white/10 text-white font-bold tracking-wider uppercase transition-colors border border-white/10"
                >
                  View in Dashboard
                </button>
              </motion.div>
            )}
          </div>
        </motion.div>
      </div>
    </AnimatePresence>
  );
}
