import { Link, useParams } from "react-router";
import { motion } from "motion/react";
import { ArrowLeft, CheckCircle2, CircleDashed, Download, FileText, MessageCircleMore, ShieldAlert, UserRound, ArrowRight } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { fetchLatestBooking, fetchTransactions, type LatestBookingResponse, type TransactionResponse } from "../lib/backend";
import { authEventName, getAuthSession, type AuthSession } from "../lib/auth";

type TimelineItem = {
  key: string;
  label: string;
  detail: string;
  status: "complete" | "current" | "upcoming" | "failed";
};

type TrackingViewModel = {
  id: string;
  title: string;
  type: string;
  asset: string;
  network: string;
  amount: string;
  fiatAmount: string;
  status: "initialized" | "awaiting" | "confirmed" | "failed";
  txHash: string;
  walletAddress: string;
  timeline: TimelineItem[];
  documents: { name: string; type: string }[];
  manager: {
    name: string;
    role: string;
    email: string;
    telegram: string;
  };
};

function buildTimeline(status: TrackingViewModel["status"]): TimelineItem[] {
  if (status === "failed") {
    return [
      { key: "initialized", label: "Payment initialized", detail: "Booking and wallet instructions were created successfully.", status: "complete" },
      { key: "awaiting", label: "Confirmation check failed", detail: "The submitted transaction did not satisfy the backend verification checks.", status: "failed" },
      { key: "confirmed", label: "Payment confirmed", detail: "Treasury will keep this order on hold until the payment is resubmitted successfully.", status: "upcoming" },
    ];
  }

  return [
    { key: "initialized", label: "Payment initialized", detail: "Wallet instructions are ready for the selected settlement route.", status: "complete" },
    { key: "awaiting", label: "Awaiting confirmations", detail: "The backend is monitoring blockchain confirmations for the submitted transaction hash.", status: status === "initialized" ? "upcoming" : status === "awaiting" ? "current" : "complete" },
    { key: "confirmed", label: "Payment confirmed", detail: "Treasury reconciliation completed and the booking can move forward operationally.", status: status === "confirmed" ? "complete" : "upcoming" },
  ];
}

function buildFromLatestBooking(booking: LatestBookingResponse): TrackingViewModel {
  const normalizedStatus = (booking.status as TrackingViewModel["status"]) || "initialized";
  return {
    id: booking.booking_id,
    title: "Flight Booking",
    type: "Booking",
    asset: booking.payment_method.toUpperCase(),
    network: booking.payment_method.toUpperCase(),
    amount: booking.total_price.toFixed(2),
    fiatAmount: `$${booking.total_price.toFixed(2)}`,
    status: normalizedStatus,
    txHash: "Pending submission",
    walletAddress: "Assigned during payment initialization",
    timeline: buildTimeline(normalizedStatus),
    documents: [
      { name: "BookingSummary.pdf", type: "Booking summary" },
      { name: "PaymentInstructions.pdf", type: "Settlement instructions" },
    ],
    manager: {
      name: "AeroLedger Treasury Desk",
      role: "Settlement Operations",
      email: "support@aeroledger.com",
      telegram: "@aeroledger_support",
    },
  };
}

function buildFromTransaction(transaction: TransactionResponse): TrackingViewModel {
  const normalizedStatus = (transaction.status as TrackingViewModel["status"]) || "awaiting";
  return {
    id: transaction.id,
    title: transaction.type.replaceAll("_", " "),
    type: "Transaction",
    asset: transaction.asset,
    network: transaction.asset,
    amount: `${transaction.amount.toFixed(2)} ${transaction.asset}`,
    fiatAmount: "Recorded on backend ledger",
    status: normalizedStatus,
    txHash: transaction.tx_hash,
    walletAddress: "Stored in payment initialization record",
    timeline: buildTimeline(normalizedStatus),
    documents: [
      { name: "LedgerTransaction.txt", type: "Transaction reference" },
      { name: "ComplianceSnapshot.txt", type: "Verification state" },
    ],
    manager: {
      name: "AeroLedger Treasury Desk",
      role: "Transaction Monitoring",
      email: "support@aeroledger.com",
      telegram: "@aeroledger_support",
    },
  };
}

export default function OrderTracking() {
  const { id } = useParams();
  const [session, setSession] = useState<AuthSession | null>(null);
  const [latestBooking, setLatestBooking] = useState<LatestBookingResponse | null>(null);
  const [transactions, setTransactions] = useState<TransactionResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const syncAuth = () => setSession(getAuthSession());
    syncAuth();
    window.addEventListener(authEventName(), syncAuth);
    return () => window.removeEventListener(authEventName(), syncAuth);
  }, []);

  useEffect(() => {
    let active = true;

    async function loadTrackingData() {
      setLoading(true);
      setError(null);
      setLatestBooking(null);
      setTransactions([]);

      if (!session) {
        setLoading(false);
        return;
      }

      try {
        const [bookingResult, transactionsResult] = await Promise.allSettled([
          fetchLatestBooking(session.token),
          fetchTransactions(session.token, session.id),
        ]);

        if (!active) return;

        if (bookingResult.status === "fulfilled") {
          setLatestBooking(bookingResult.value);
        }

        if (transactionsResult.status === "fulfilled") {
          setTransactions(transactionsResult.value);
        }

        if (bookingResult.status === "rejected" && transactionsResult.status === "rejected") {
          throw bookingResult.reason;
        }
      } catch (err) {
        if (!active) return;
        setError(err instanceof Error ? err.message : "Unable to load tracking details.");
      } finally {
        if (active) {
          setLoading(false);
        }
      }
    }

    loadTrackingData();
    return () => {
      active = false;
    };
  }, [session]);

  const order = useMemo(() => {
    if (!id) {
      return null;
    }

    if (latestBooking && latestBooking.booking_id === id) {
      return buildFromLatestBooking(latestBooking);
    }

    const matchingTransaction = transactions.find((transaction) => transaction.id === id);
    if (matchingTransaction) {
      return buildFromTransaction(matchingTransaction);
    }

    return latestBooking ? buildFromLatestBooking(latestBooking) : null;
  }, [id, latestBooking, transactions]);

  if (!session) {
    return (
      <div className="flex min-h-[60vh] flex-col items-center justify-center px-6 text-center">
        <h1 className="text-3xl font-bold text-white">Sign in to open order tracking</h1>
        <p className="mt-3 text-slate-400">Backend tracking data is protected per user account, so this page needs an active session.</p>
        <Link to="/signin" className="mt-6 inline-flex items-center gap-2 rounded-2xl bg-amber-500 px-6 py-4 font-semibold text-black hover:bg-amber-400">
          Sign In <ArrowRight className="h-4 w-4" />
        </Link>
      </div>
    );
  }

  if (loading) {
    return <div className="max-w-7xl mx-auto px-6 py-20 text-center text-slate-400">Loading tracking details...</div>;
  }

  if (error) {
    return (
      <div className="max-w-4xl mx-auto px-6 py-20 text-center">
        <h1 className="text-3xl font-bold text-white">Unable to load tracking</h1>
        <p className="mt-3 text-slate-400">{error}</p>
        <Link to="/transactions" className="mt-6 inline-flex items-center gap-2 text-amber-500 hover:text-amber-400">
          <ArrowLeft className="h-4 w-4" /> Back to Transactions
        </Link>
      </div>
    );
  }

  if (!order) {
    return (
      <div className="flex min-h-[60vh] flex-col items-center justify-center px-6 text-center">
        <h1 className="text-3xl font-bold text-white">Order not found</h1>
        <p className="mt-3 text-slate-400">The requested backend booking or transaction reference is unavailable for this account.</p>
        <Link to="/transactions" className="mt-6 inline-flex items-center gap-2 text-amber-500 hover:text-amber-400">
          <ArrowLeft className="h-4 w-4" /> Back to Transactions
        </Link>
      </div>
    );
  }

  const statusTone = {
    initialized: "text-sky-400 border-sky-500/20 bg-sky-500/10",
    awaiting: "text-amber-400 border-amber-500/20 bg-amber-500/10",
    confirmed: "text-emerald-400 border-emerald-500/20 bg-emerald-500/10",
    failed: "text-rose-400 border-rose-500/20 bg-rose-500/10",
  }[order.status];

  return (
    <div className="max-w-7xl mx-auto px-6 py-8">
      <Link to="/transactions" className="inline-flex items-center gap-2 text-slate-400 hover:text-white transition-colors mb-8 text-sm font-medium">
        <ArrowLeft className="w-4 h-4" /> Back to Transactions
      </Link>

      <div className="grid grid-cols-1 xl:grid-cols-[1fr_340px] gap-8">
        <div>
          <div className="rounded-[2rem] border border-white/8 bg-[#0A0A0E] p-8 shadow-2xl shadow-black/40">
            <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-6 border-b border-white/10 pb-8">
              <div>
                <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Order Reference</p>
                <h1 className="mt-2 text-3xl md:text-4xl font-bold text-white">{order.id}</h1>
                <p className="mt-3 text-slate-400">{order.title} - {order.type}</p>
              </div>
              <div className={`inline-flex w-fit items-center gap-2 rounded-full border px-4 py-2 text-sm font-semibold uppercase tracking-[0.24em] ${statusTone}`}>
                {order.status}
              </div>
            </div>

            <div className="mt-8 grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="rounded-2xl border border-white/8 bg-white/[0.03] p-5">
                <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Asset</p>
                <p className="mt-3 text-lg font-semibold text-white">{order.asset}</p>
                <p className="mt-1 text-sm text-slate-500">{order.network}</p>
              </div>
              <div className="rounded-2xl border border-white/8 bg-white/[0.03] p-5">
                <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Amount</p>
                <p className="mt-3 text-lg font-semibold text-white">{order.amount}</p>
                <p className="mt-1 text-sm text-slate-500">{order.fiatAmount}</p>
              </div>
              <div className="rounded-2xl border border-white/8 bg-white/[0.03] p-5">
                <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Transaction Hash</p>
                <p className="mt-3 text-sm font-mono text-white break-all">{order.txHash}</p>
              </div>
            </div>

            <div className="mt-10">
              <div className="flex items-center justify-between gap-4 mb-6">
                <h2 className="text-2xl font-bold text-white">Timeline</h2>
                <p className="text-sm text-slate-500">Wallet: {order.walletAddress}</p>
              </div>

              <div className="space-y-5">
                {order.timeline.map((item, index) => {
                  const icon = item.status === "complete"
                    ? <CheckCircle2 className="h-5 w-5 text-emerald-400" />
                    : item.status === "failed"
                      ? <ShieldAlert className="h-5 w-5 text-rose-400" />
                      : <CircleDashed className="h-5 w-5 text-amber-400" />;

                  return (
                    <motion.div
                      key={item.key}
                      initial={{ opacity: 0, y: 14 }}
                      animate={{ opacity: 1, y: 0 }}
                      transition={{ delay: index * 0.08 }}
                      className="relative rounded-3xl border border-white/8 bg-white/[0.03] p-5"
                    >
                      <div className="flex gap-4">
                        <div className="mt-1">{icon}</div>
                        <div>
                          <p className="text-sm font-semibold uppercase tracking-[0.24em] text-slate-400">{item.label}</p>
                          <p className="mt-3 text-sm leading-6 text-slate-300">{item.detail}</p>
                        </div>
                      </div>
                    </motion.div>
                  );
                })}
              </div>
            </div>
          </div>
        </div>

        <div className="space-y-6">
          <div className="rounded-3xl border border-white/8 bg-[#0A0A0E] p-6">
            <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Documents</p>
            <div className="mt-5 space-y-3">
              {order.documents.map((document) => (
                <button
                  key={document.name}
                  type="button"
                  className="flex w-full items-center justify-between rounded-2xl border border-white/8 bg-white/[0.03] px-4 py-4 text-left hover:bg-white/[0.05] transition-colors"
                >
                  <div className="flex items-center gap-3">
                    <FileText className="h-4 w-4 text-amber-500" />
                    <div>
                      <p className="text-sm font-medium text-white">{document.name}</p>
                      <p className="text-xs text-slate-500">{document.type}</p>
                    </div>
                  </div>
                  <Download className="h-4 w-4 text-slate-500" />
                </button>
              ))}
            </div>
          </div>

          <div className="rounded-3xl border border-white/8 bg-[#0A0A0E] p-6">
            <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Transaction Manager</p>
            <div className="mt-5 rounded-2xl border border-white/8 bg-white/[0.03] p-5">
              <div className="flex items-center gap-3">
                <div className="flex h-11 w-11 items-center justify-center rounded-full bg-amber-500/10">
                  <UserRound className="h-5 w-5 text-amber-400" />
                </div>
                <div>
                  <p className="text-base font-semibold text-white">{order.manager.name}</p>
                  <p className="text-sm text-slate-500">{order.manager.role}</p>
                </div>
              </div>
              <div className="mt-5 space-y-2 text-sm text-slate-300">
                <p>{order.manager.email}</p>
                <p>{order.manager.telegram}</p>
              </div>
              <button type="button" className="mt-5 inline-flex items-center gap-2 rounded-2xl border border-white/10 bg-white/[0.03] px-4 py-3 text-sm font-medium text-white hover:bg-white/[0.06] transition-colors">
                <MessageCircleMore className="h-4 w-4 text-amber-500" /> Contact Manager
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
