import { motion } from "motion/react";
import { Link, useNavigate, useSearchParams } from "react-router";
import { CheckCircle2, ArrowRight, ScanLine, ShieldAlert, Copy, FileDigit, UserRound, Upload } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { getCartItems, type CartItem } from "../lib/commerce";
import { FALLBACK_PAYMENT_OPTIONS, fetchPaymentOptions, type PaymentOption } from "../lib/paymentOptions";
import { confirmPayment, createBooking, initPayment, uploadPaymentProof } from "../lib/backend";
import { authEventName, getAuthSession, type AuthSession } from "../lib/auth";

type CheckoutStatus = "idle" | "initialized" | "awaiting" | "confirmed";

const checkoutSteps = ["Review", "Client Details", "Crypto Payment", "Tracking"];

function parseFiatAmount(value: string) {
  const numeric = Number(value.replace(/[^0-9.]+/g, ""));
  return Number.isFinite(numeric) ? numeric : 0;
}

export default function Checkout() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const itemId = searchParams.get("itemId");
  const [cartItems, setCartItems] = useState<CartItem[]>([]);
  const [currentStep, setCurrentStep] = useState(0);
  const [paymentStatus, setPaymentStatus] = useState<CheckoutStatus>("idle");
  const [paymentOptions, setPaymentOptions] = useState<PaymentOption[]>(FALLBACK_PAYMENT_OPTIONS);
  const [selectedMethod, setSelectedMethod] = useState("btc");
  const [copied, setCopied] = useState(false);
  const [transactionHash, setTransactionHash] = useState("");
  const [authSession, setAuthSession] = useState<AuthSession | null>(null);
  const [walletAddress, setWalletAddress] = useState<string | null>(null);
  const [quotedAmount, setQuotedAmount] = useState<number | null>(null);
  const [bookingId, setBookingId] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [proofUploading, setProofUploading] = useState(false);
  const [proofFile, setProofFile] = useState<File | null>(null);
  const [proofMessage, setProofMessage] = useState<string | null>(null);
  const [clientForm, setClientForm] = useState({
    fullName: "",
    email: "",
    phone: "",
    notes: "",
  });

  useEffect(() => {
    setCartItems(getCartItems());
    const syncAuth = () => setAuthSession(getAuthSession());
    syncAuth();
    window.addEventListener(authEventName(), syncAuth);

    let active = true;
    fetchPaymentOptions().then((options) => {
      if (!active) return;
      setPaymentOptions(options);
      const firstAvailable = options.find((option) => option.available) ?? options[0];
      if (firstAvailable) {
        setSelectedMethod(firstAvailable.method);
      }
    });

    return () => {
      active = false;
      window.removeEventListener(authEventName(), syncAuth);
    };
  }, []);

  const selectedItem = useMemo(() => {
    return cartItems.find((item) => item.id === itemId) ?? cartItems[0];
  }, [cartItems, itemId]);

  const visibleOptions = useMemo(() => {
    const available = paymentOptions.filter((option) => option.available);
    return available.length > 0 ? available : paymentOptions;
  }, [paymentOptions]);

  const selectedOption = visibleOptions.find((option) => option.method === selectedMethod) ?? visibleOptions[0] ?? FALLBACK_PAYMENT_OPTIONS[0];
  const canUseBackendCheckout = Boolean(selectedItem);
  const orderLink = bookingId ? `/orders/${bookingId}` : "/transactions";

  async function ensurePaymentInitialized() {
    if (!authSession) {
      setFormError("Sign in first to create a booking and initialize payment.");
      return null;
    }

    if (!clientForm.fullName.trim() || !clientForm.email.trim()) {
      setFormError("Add at least the client name and email before continuing.");
      setCurrentStep(1);
      return null;
    }

    if (!canUseBackendCheckout || !selectedItem) {
      setFormError("Choose an item before continuing.");
      return null;
    }

    if (bookingId) {
      return bookingId;
    }

    const createdBooking = await createBooking(authSession.token, {
      user_id: authSession.id,
      flight_id: selectedItem.kind === "flight" ? selectedItem.backendFlightId : undefined,
      base_price: parseFiatAmount(selectedItem.fiatPrice),
      payment_method: selectedMethod,
    });

    const newBookingId = createdBooking.booking_id;
    setBookingId(newBookingId);

    const payment = await initPayment(authSession.token, {
      booking_id: newBookingId,
      method: selectedMethod,
    });

    setWalletAddress(payment.wallet_address);
    setQuotedAmount(payment.amount);
    setPaymentStatus("initialized");
    return newBookingId;
  }

  if (!selectedItem) {
    return (
      <div className="max-w-4xl mx-auto px-6 py-20 text-center">
        <h1 className="text-3xl font-bold text-white">No cart item selected</h1>
        <p className="mt-3 text-slate-400">Choose an item from your cart before starting payment verification.</p>
        <Link to="/cart" className="mt-6 inline-flex items-center gap-2 rounded-2xl bg-amber-500 px-6 py-4 font-semibold text-black hover:bg-amber-400">
          Go to Cart <ArrowRight className="h-4 w-4" />
        </Link>
      </div>
    );
  }

  const displayedWalletAddress = walletAddress ?? selectedOption.wallet_address ?? "Wallet address becomes available once treasury initializes this route.";

  const handleInitializePayment = async () => {
    setFormError(null);
    setProofMessage(null);
    setSubmitting(true);
    try {
      const ensuredBookingId = await ensurePaymentInitialized();
      if (!ensuredBookingId) {
        return;
      }
      setCurrentStep(3);
    } catch (err) {
      setFormError(err instanceof Error ? err.message : "Unable to initialize payment.");
    } finally {
      setSubmitting(false);
    }
  };

  const handleSubmitHash = async () => {
    setFormError(null);
    setProofMessage(null);
    setSubmitting(true);

    try {
      const ensuredBookingId = await ensurePaymentInitialized();
      if (!ensuredBookingId) {
        return;
      }

      if (!transactionHash.trim()) {
        setFormError("Enter a transaction hash before submitting.");
        return;
      }

      setPaymentStatus("awaiting");
      await confirmPayment(authSession!.token, {
        booking_id: ensuredBookingId,
        tx_hash: transactionHash.trim(),
      });
      setPaymentStatus("confirmed");
      setCurrentStep(3);
    } catch (err) {
      setPaymentStatus("initialized");
      setFormError(err instanceof Error ? err.message : "Unable to confirm payment.");
    } finally {
      setSubmitting(false);
    }
  };

  const handleUploadProof = async () => {
    setFormError(null);
    setProofMessage(null);

    if (!proofFile) {
      setFormError("Choose a payment screenshot before uploading.");
      return;
    }

    if (!proofFile.type.startsWith("image/")) {
      setFormError("Payment proof must be an image file.");
      return;
    }

    setProofUploading(true);
    try {
      const ensuredBookingId = await ensurePaymentInitialized();
      if (!ensuredBookingId) {
        return;
      }

      const result = await uploadPaymentProof(authSession!.token, ensuredBookingId, proofFile);
      setProofMessage(result.message);
      setCurrentStep(3);
    } catch (err) {
      setFormError(err instanceof Error ? err.message : "Unable to upload payment proof.");
    } finally {
      setProofUploading(false);
    }
  };

  const handleCopy = async () => {
    if (!displayedWalletAddress || displayedWalletAddress.includes("Wallet address becomes available")) return;
    await navigator.clipboard.writeText(displayedWalletAddress);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1800);
  };

  const statusItems = [
    {
      key: "initialized",
      label: "Initialized",
      detail: "Wallet instructions generated for the selected network.",
      active: paymentStatus === "initialized" || paymentStatus === "awaiting" || paymentStatus === "confirmed",
    },
    {
      key: "awaiting",
      label: "Awaiting",
      detail: "Transaction hash submitted and blockchain confirmations in progress.",
      active: paymentStatus === "awaiting" || paymentStatus === "confirmed",
    },
    {
      key: "confirmed",
      label: "Confirmed",
      detail: "Treasury and reconciliation checks are complete.",
      active: paymentStatus === "confirmed",
    },
  ];

  return (
    <div className="max-w-7xl mx-auto px-6 py-8">
      <div className="mb-10 text-center lg:text-left">
        <h1 className="text-4xl md:text-5xl font-bold text-white mb-3">Crypto Checkout Flow</h1>
        <p className="text-slate-400 max-w-3xl">
          Complete booking and purchase settlement with wallet initialization, transaction hash submission, and blockchain status tracking.
        </p>
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-[1fr_360px] gap-8">
        <div className="rounded-[2rem] border border-white/8 bg-[#0A0A0E] p-6 md:p-8 shadow-2xl shadow-black/40">
          <div className="flex flex-wrap gap-3 border-b border-white/10 pb-6 mb-8">
            {checkoutSteps.map((step, index) => {
              const isActive = index === currentStep;
              const isComplete = index < currentStep;
              return (
                <button
                  key={step}
                  type="button"
                  onClick={() => {
                    if (index <= currentStep) setCurrentStep(index);
                  }}
                  className={`rounded-full px-4 py-2 text-sm font-medium transition-all ${
                    isActive
                      ? "bg-amber-500 text-black"
                      : isComplete
                        ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20"
                        : "bg-white/[0.04] text-slate-400 border border-white/10"
                  }`}
                >
                  {index + 1}. {step}
                </button>
              );
            })}
          </div>

          {formError && (
            <div className="mb-6 rounded-2xl border border-rose-500/20 bg-rose-500/10 px-4 py-3 text-sm text-rose-200">
              {formError}
            </div>
          )}

          {proofMessage && (
            <div className="mb-6 rounded-2xl border border-emerald-500/20 bg-emerald-500/10 px-4 py-3 text-sm text-emerald-200">
              {proofMessage}
            </div>
          )}

          {currentStep === 0 && (
            <div className="space-y-8">
              <div className="rounded-3xl border border-white/8 bg-white/[0.03] p-6">
                <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Selected item</p>
                <h2 className="mt-3 text-3xl font-bold text-white">{selectedItem.name}</h2>
                <p className="mt-2 text-slate-400">{selectedItem.subtitle}</p>
                <div className="mt-6 grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="rounded-2xl border border-white/8 bg-black/30 p-4">
                    <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Primary detail</p>
                    <p className="mt-2 text-white">{selectedItem.primaryMeta}</p>
                  </div>
                  <div className="rounded-2xl border border-white/8 bg-black/30 p-4">
                    <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Secondary detail</p>
                    <p className="mt-2 text-white">{selectedItem.secondaryMeta}</p>
                  </div>
                </div>
              </div>

              <div className="rounded-3xl border border-white/8 bg-white/[0.03] p-6">
                <h3 className="text-xl font-semibold text-white">Settlement path</h3>
                <div className="mt-5 space-y-4">
                  {[
                    "Treasury initializes payment instructions for the selected asset and network.",
                    "Client submits a blockchain transaction hash after funding.",
                    "Client can upload a payment screenshot that gets attached to the backend payment record.",
                    "AeroLedger monitors confirmations and updates status from initialized to awaiting to confirmed.",
                  ].map((text) => (
                    <div key={text} className="flex gap-3">
                      <CheckCircle2 className="mt-0.5 h-5 w-5 text-amber-500" />
                      <p className="text-sm leading-6 text-slate-300">{text}</p>
                    </div>
                  ))}
                </div>
              </div>

              <div className="flex justify-end">
                <button type="button" onClick={() => setCurrentStep(1)} className="rounded-2xl bg-amber-500 px-6 py-4 font-semibold text-black hover:bg-amber-400 transition-colors inline-flex items-center gap-2">
                  Continue to Client Details <ArrowRight className="h-4 w-4" />
                </button>
              </div>
            </div>
          )}

          {currentStep === 1 && (
            <div className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
                <label className="space-y-2">
                  <span className="text-xs uppercase tracking-[0.24em] text-slate-500">Full name</span>
                  <input value={clientForm.fullName} onChange={(e) => setClientForm({ ...clientForm, fullName: e.target.value })} className="w-full rounded-2xl border border-white/10 bg-white/[0.04] px-4 py-4 text-white outline-none focus:border-amber-500/60" placeholder="Client name" />
                </label>
                <label className="space-y-2">
                  <span className="text-xs uppercase tracking-[0.24em] text-slate-500">Email</span>
                  <input value={clientForm.email} onChange={(e) => setClientForm({ ...clientForm, email: e.target.value })} className="w-full rounded-2xl border border-white/10 bg-white/[0.04] px-4 py-4 text-white outline-none focus:border-amber-500/60" placeholder="client@aeroledger.com" />
                </label>
                <label className="space-y-2">
                  <span className="text-xs uppercase tracking-[0.24em] text-slate-500">Phone</span>
                  <input value={clientForm.phone} onChange={(e) => setClientForm({ ...clientForm, phone: e.target.value })} className="w-full rounded-2xl border border-white/10 bg-white/[0.04] px-4 py-4 text-white outline-none focus:border-amber-500/60" placeholder="+1 212 555 0100" />
                </label>
                <label className="space-y-2">
                  <span className="text-xs uppercase tracking-[0.24em] text-slate-500">Special notes</span>
                  <input value={clientForm.notes} onChange={(e) => setClientForm({ ...clientForm, notes: e.target.value })} className="w-full rounded-2xl border border-white/10 bg-white/[0.04] px-4 py-4 text-white outline-none focus:border-amber-500/60" placeholder="Arrival preferences or escrow notes" />
                </label>
              </div>

              <div className="rounded-3xl border border-white/8 bg-white/[0.03] p-6 flex items-start gap-4">
                <UserRound className="h-5 w-5 text-amber-500 mt-1" />
                <p className="text-sm leading-6 text-slate-300">Client identity details remain attached to the settlement record so operations, treasury, and the transaction manager can reconcile payment and release documents quickly.</p>
              </div>

              {!authSession && (
                <div className="rounded-3xl border border-amber-500/20 bg-amber-500/10 p-5 text-sm text-amber-100/90">
                  Sign in before initializing payment so the booking can be created against your backend account.
                </div>
              )}

              <div className="flex justify-between gap-4">
                <button type="button" onClick={() => setCurrentStep(0)} className="rounded-2xl border border-white/10 px-6 py-4 text-slate-300 hover:bg-white/[0.04]">
                  Back
                </button>
                <button type="button" onClick={() => setCurrentStep(2)} className="rounded-2xl bg-amber-500 px-6 py-4 font-semibold text-black hover:bg-amber-400 inline-flex items-center gap-2">
                  Continue to Payment <ArrowRight className="h-4 w-4" />
                </button>
              </div>
            </div>
          )}

          {currentStep === 2 && (
            <div className="space-y-8">
              <div>
                <label className="text-xs uppercase tracking-[0.24em] text-slate-500 mb-3 block">Choose payment network</label>
                <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
                  {visibleOptions.map((option) => {
                    const active = option.method === selectedOption.method;
                    return (
                      <button key={option.method} type="button" onClick={() => setSelectedMethod(option.method)} className={`rounded-2xl border px-4 py-4 text-left transition-all ${active ? "border-amber-500/40 bg-amber-500/10" : "border-white/10 bg-white/[0.03] hover:bg-white/[0.05]"}`}>
                        <p className="text-sm font-semibold text-white">{option.token}</p>
                        <p className="mt-1 text-xs text-slate-400">{option.chain}</p>
                      </button>
                    );
                  })}
                </div>
              </div>

              <div className="grid grid-cols-1 lg:grid-cols-[220px_1fr] gap-8 items-start">
                <div className="rounded-3xl bg-white p-4 h-[220px] flex items-center justify-center relative">
                  <ScanLine className="w-full h-full text-slate-200" />
                  <div className="absolute inset-0 m-auto flex h-16 w-16 items-center justify-center rounded-full bg-white text-amber-500 font-bold shadow-lg">
                    {selectedOption.token}
                  </div>
                </div>

                <div className="space-y-5">
                  <div className="rounded-2xl border border-white/8 bg-white/[0.03] p-5">
                    <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Wallet route</p>
                    <p className="mt-3 text-lg font-semibold text-white">{selectedOption.display_name}</p>
                    <p className="mt-2 break-all font-mono text-sm text-slate-300">{displayedWalletAddress}</p>
                    <button type="button" onClick={handleCopy} disabled={!walletAddress && !selectedOption.wallet_address} className="mt-4 inline-flex items-center gap-2 rounded-2xl border border-white/10 px-4 py-3 text-sm text-white hover:bg-white/[0.05] disabled:opacity-40">
                      {copied ? <CheckCircle2 className="h-4 w-4 text-emerald-400" /> : <Copy className="h-4 w-4 text-amber-500" />} Copy address
                    </button>
                  </div>

                  <div className="rounded-2xl border border-white/8 bg-white/[0.03] p-5">
                    <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Quoted settlement</p>
                    <p className="mt-3 text-2xl font-bold text-white">{selectedItem.cryptoPrice}</p>
                    <p className="mt-1 text-sm text-slate-500">Approx. {selectedItem.fiatPrice}</p>
                    {quotedAmount !== null && (
                      <p className="mt-2 text-sm text-emerald-400">Backend quote initialized: {quotedAmount.toFixed(2)}</p>
                    )}
                  </div>
                </div>
              </div>

              <div className="rounded-2xl border border-amber-500/20 bg-amber-500/10 p-5 flex gap-3">
                <ShieldAlert className="h-5 w-5 text-amber-400 mt-0.5" />
                <p className="text-sm leading-6 text-amber-100/80">
                  {canUseBackendCheckout
                    ? "You can initialize payment here, or go straight to hash submission and the app will create the booking/payment automatically before sending the hash or proof image."
                    : "Choose an item before continuing with checkout."}
                </p>
              </div>

              <div className="flex flex-col md:flex-row gap-4 justify-between">
                <button type="button" onClick={() => setCurrentStep(1)} className="rounded-2xl border border-white/10 px-6 py-4 text-slate-300 hover:bg-white/[0.04]">
                  Back
                </button>
                <div className="flex flex-col sm:flex-row gap-4">
                  {!authSession ? (
                    <button type="button" onClick={() => navigate("/signin")} className="rounded-2xl border border-white/10 px-6 py-4 text-white hover:bg-white/[0.04]">
                      Sign In to Initialize
                    </button>
                  ) : (
                    <button type="button" onClick={handleInitializePayment} disabled={submitting || proofUploading || !canUseBackendCheckout} className="rounded-2xl border border-white/10 px-6 py-4 text-white hover:bg-white/[0.04] disabled:opacity-50">
                      {submitting ? "Initializing..." : bookingId ? "Refresh Payment Route" : "Initialize Payment"}
                    </button>
                  )}
                  <button type="button" onClick={() => setCurrentStep(3)} className="rounded-2xl bg-amber-500 px-6 py-4 font-semibold text-black hover:bg-amber-400 inline-flex items-center gap-2">
                    Go to Hash Submission <ArrowRight className="h-4 w-4" />
                  </button>
                </div>
              </div>
            </div>
          )}

          {currentStep === 3 && (
            <div className="space-y-8">
              <div className="rounded-3xl border border-white/8 bg-white/[0.03] p-6">
                <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Transaction hash submission</p>
                <div className="mt-4 flex flex-col lg:flex-row gap-4">
                  <div className="flex-1">
                    <input value={transactionHash} onChange={(e) => setTransactionHash(e.target.value)} className="w-full rounded-2xl border border-white/10 bg-white/[0.04] px-4 py-4 text-white outline-none focus:border-amber-500/60" placeholder="Paste your blockchain transaction hash" />
                  </div>
                  <button type="button" onClick={handleSubmitHash} disabled={submitting || proofUploading} className="rounded-2xl bg-amber-500 px-6 py-4 font-semibold text-black hover:bg-amber-400 inline-flex items-center gap-2 disabled:opacity-50">
                    {submitting ? "Submitting..." : "Submit Hash"} <FileDigit className="h-4 w-4" />
                  </button>
                </div>
                {!bookingId && (
                  <p className="mt-3 text-sm text-amber-300">
                    No booking exists yet. When you submit the hash, AeroLedger will create the booking and initialize payment automatically first.
                  </p>
                )}
              </div>

              <div className="rounded-3xl border border-white/8 bg-white/[0.03] p-6">
                <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Payment proof screenshot</p>
                <div className="mt-4 flex flex-col gap-4 lg:flex-row lg:items-center">
                  <label className="flex-1 rounded-2xl border border-dashed border-white/15 bg-white/[0.03] px-4 py-4 text-sm text-slate-300 cursor-pointer hover:bg-white/[0.05]">
                    <input
                      type="file"
                      accept="image/*"
                      className="hidden"
                      onChange={(event) => {
                        const file = event.target.files?.[0] ?? null;
                        setProofFile(file);
                        setProofMessage(null);
                      }}
                    />
                    {proofFile ? proofFile.name : "Choose a screenshot or payment receipt image"}
                  </label>
                  <button type="button" onClick={handleUploadProof} disabled={proofUploading || submitting} className="rounded-2xl border border-white/10 px-6 py-4 text-white hover:bg-white/[0.04] inline-flex items-center justify-center gap-2 disabled:opacity-50">
                    <Upload className="h-4 w-4 text-amber-400" />
                    {proofUploading ? "Uploading..." : "Upload Proof"}
                  </button>
                </div>
                <p className="mt-3 text-sm text-slate-500">
                  Uploading a screenshot stores it on the backend and attaches it to the payment record for this booking.
                </p>
              </div>

              <div className="space-y-4">
                {statusItems.map((item) => (
                  <div key={item.key} className={`rounded-2xl border p-5 ${item.active ? "border-amber-500/30 bg-amber-500/10" : "border-white/8 bg-white/[0.03]"}`}>
                    <div className="flex items-center justify-between gap-4">
                      <div>
                        <p className="text-sm font-semibold uppercase tracking-[0.24em] text-white">{item.label}</p>
                        <p className="mt-2 text-sm text-slate-300">{item.detail}</p>
                      </div>
                      {item.active ? <CheckCircle2 className="h-5 w-5 text-emerald-400" /> : <div className="h-5 w-5 rounded-full border border-white/20" />}
                    </div>
                  </div>
                ))}
              </div>

              <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4 border-t border-white/10 pt-4">
                <p className="text-sm text-slate-400">Current state: <span className="font-semibold text-white">{paymentStatus === "idle" ? "waiting for initialization" : paymentStatus}</span></p>
                {paymentStatus === "confirmed" ? (
                  <Link to={orderLink} className="rounded-2xl bg-emerald-500 px-6 py-4 font-semibold text-black hover:bg-emerald-400 inline-flex items-center gap-2">
                    Open Order Tracking <ArrowRight className="h-4 w-4" />
                  </Link>
                ) : (
                  <button type="button" onClick={() => setCurrentStep(2)} className="rounded-2xl border border-white/10 px-6 py-4 text-slate-300 hover:bg-white/[0.04]">
                    Back to Payment Setup
                  </button>
                )}
              </div>
            </div>
          )}
        </div>

        <div className="space-y-6">
          <div className="rounded-3xl border border-white/8 bg-[#0A0A0E] p-6">
            <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Order Summary</p>
            <h2 className="mt-3 text-2xl font-bold text-white">{selectedItem.name}</h2>
            <p className="mt-2 text-sm text-slate-400">{selectedItem.subtitle}</p>
            <div className="mt-6 space-y-4">
              <div className="flex justify-between text-sm">
                <span className="text-slate-500">Category</span>
                <span className="text-white">{selectedItem.category}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-slate-500">Settlement</span>
                <span className="text-white">{selectedItem.cryptoPrice}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-slate-500">Fiat estimate</span>
                <span className="text-white">{selectedItem.fiatPrice}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-slate-500">Backend flow</span>
                <span className="text-white">{canUseBackendCheckout ? "Available" : "Unavailable"}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-slate-500">Booking</span>
                <span className="text-white">{bookingId ?? "Will be created automatically"}</span>
              </div>
            </div>
          </div>

          <div className="rounded-3xl border border-white/8 bg-[#0A0A0E] p-6">
            <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Status Progression</p>
            <div className="mt-6 space-y-5">
              {statusItems.map((item) => (
                <div key={item.key} className="flex gap-3">
                  <div className={`mt-1 h-3 w-3 rounded-full ${item.active ? "bg-amber-500" : "bg-white/10"}`} />
                  <div>
                    <p className="text-sm font-medium text-white">{item.label}</p>
                    <p className="mt-1 text-xs leading-5 text-slate-500">{item.detail}</p>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {!authSession && (
            <div className="rounded-3xl border border-amber-500/20 bg-amber-500/10 p-6">
              <p className="text-sm text-amber-100/90">Protected booking, payment initialization, payment-proof upload, and transaction history use the backend API. Sign in to use the live flow.</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}


