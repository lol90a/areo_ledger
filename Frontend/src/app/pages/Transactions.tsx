import { motion } from "motion/react";
import { ArrowDownRight, ArrowUpRight, Search, Download, Filter, ExternalLink, ArrowRight } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router";

import { fetchTransactions, type TransactionResponse } from "../lib/backend";
import { authEventName, getAuthSession, type AuthSession } from "../lib/auth";

type UiTransaction = TransactionResponse & {
  title: string;
};

function formatAmount(amount: number, asset: string) {
  return `${amount.toFixed(2)} ${asset}`;
}

export default function Transactions() {
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState("all");
  const [transactions, setTransactions] = useState<UiTransaction[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [session, setSession] = useState<AuthSession | null>(null);

  useEffect(() => {
    const syncAuth = () => setSession(getAuthSession());
    syncAuth();
    window.addEventListener(authEventName(), syncAuth);
    return () => window.removeEventListener(authEventName(), syncAuth);
  }, []);

  useEffect(() => {
    let active = true;

    async function loadTransactions() {
      setLoading(true);
      setError(null);
      setTransactions([]);

      if (!session) {
        setLoading(false);
        return;
      }

      try {
        const response = await fetchTransactions(session.token, session.id);
        if (!active) return;
        setTransactions(
          response.map((transaction) => ({
            ...transaction,
            title: transaction.type.replaceAll("_", " "),
          })),
        );
      } catch (err) {
        if (!active) return;
        setError(err instanceof Error ? err.message : "Unable to load transaction history.");
      } finally {
        if (active) {
          setLoading(false);
        }
      }
    }

    loadTransactions();
    return () => {
      active = false;
    };
  }, [session]);

  const filtered = useMemo(() => {
    return transactions.filter((transaction) => {
      const matchesQuery = [transaction.id, transaction.title, transaction.tx_hash, transaction.asset]
        .join(" ")
        .toLowerCase()
        .includes(query.toLowerCase());
      const matchesStatus = statusFilter === "all" || transaction.status === statusFilter;
      return matchesQuery && matchesStatus;
    });
  }, [transactions, query, statusFilter]);

  const exportCsv = () => {
    const header = ["Transaction ID", "Date", "Type", "Asset", "Amount", "Status", "Tx Hash"];
    const rows = filtered.map((transaction) => [transaction.id, transaction.date, transaction.type, transaction.asset, transaction.amount.toString(), transaction.status, transaction.tx_hash]);
    const csv = [header, ...rows].map((row) => row.map((cell) => `"${cell.replaceAll('"', '""')}"`).join(",")).join("\n");
    const blob = new Blob([csv], { type: "text/csv;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = "aeroledger-transactions.csv";
    link.click();
    URL.revokeObjectURL(url);
  };

  if (!session) {
    return (
      <div className="max-w-4xl mx-auto px-6 py-20 text-center">
        <h1 className="text-3xl font-bold text-white">Sign in to view transaction history</h1>
        <p className="mt-3 text-slate-400">The backend transaction ledger is protected per user account. Sign in to load your live settlement history.</p>
        <Link to="/signin" className="mt-6 inline-flex items-center gap-2 rounded-2xl bg-amber-500 px-6 py-4 font-semibold text-black hover:bg-amber-400">
          Sign In <ArrowRight className="h-4 w-4" />
        </Link>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-6 py-8">
      <div className="flex flex-col md:flex-row justify-between items-start md:items-end mb-10 gap-6">
        <div>
          <h1 className="text-3xl md:text-4xl font-bold text-white mb-2">Transaction History</h1>
          <p className="text-slate-400">Search settlement activity, monitor backend status progression, and export payment records for audit review.</p>
        </div>
        <div className="flex flex-wrap items-center gap-4 w-full md:w-auto">
          <div className="relative flex-1 md:w-72">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              type="text"
              placeholder="Search ID, asset, or hash..."
              className="w-full pl-11 pr-4 py-2.5 rounded-full bg-white/5 border border-white/10 text-white placeholder-slate-500 focus:outline-none focus:border-amber-500 transition-colors text-sm"
            />
          </div>
          <div className="flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-4 py-2.5">
            <Filter className="w-4 h-4 text-slate-300" />
            <select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value)} className="bg-transparent text-sm text-white outline-none">
              <option value="all" className="bg-black">All statuses</option>
              <option value="initialized" className="bg-black">Initialized</option>
              <option value="awaiting" className="bg-black">Awaiting</option>
              <option value="confirmed" className="bg-black">Confirmed</option>
              <option value="failed" className="bg-black">Failed</option>
            </select>
          </div>
          <button onClick={exportCsv} className="p-2.5 rounded-full bg-white/5 border border-white/10 hover:bg-white/10 transition-colors flex items-center gap-2 px-4 text-sm text-white font-medium">
            <Download className="w-4 h-4" /> Export CSV
          </button>
        </div>
      </div>

      {error && (
        <div className="mb-6 rounded-2xl border border-rose-500/20 bg-rose-500/10 px-4 py-3 text-sm text-rose-200">
          {error}
        </div>
      )}

      <div className="w-full overflow-x-auto rounded-3xl bg-[#0A0A0E] border border-white/5">
        <table className="w-full text-left border-collapse min-w-[920px]">
          <thead>
            <tr className="border-b border-white/5 bg-white/5">
              <th className="py-4 px-6 text-xs uppercase tracking-wider font-semibold text-slate-500">Transaction ID</th>
              <th className="py-4 px-6 text-xs uppercase tracking-wider font-semibold text-slate-500">Date</th>
              <th className="py-4 px-6 text-xs uppercase tracking-wider font-semibold text-slate-500">Type</th>
              <th className="py-4 px-6 text-xs uppercase tracking-wider font-semibold text-slate-500">Amount</th>
              <th className="py-4 px-6 text-xs uppercase tracking-wider font-semibold text-slate-500">Asset</th>
              <th className="py-4 px-6 text-xs uppercase tracking-wider font-semibold text-slate-500">Status</th>
              <th className="py-4 px-6 text-xs uppercase tracking-wider font-semibold text-slate-500">Tracking</th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              <tr>
                <td colSpan={7} className="py-10 px-6 text-center text-slate-400">Loading transactions...</td>
              </tr>
            ) : filtered.length === 0 ? (
              <tr>
                <td colSpan={7} className="py-10 px-6 text-center text-slate-400">No transactions matched your current filters.</td>
              </tr>
            ) : (
              filtered.map((transaction, idx) => (
                <motion.tr
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: idx * 0.04 }}
                  key={transaction.id}
                  className="border-b border-white/5 hover:bg-white/5 transition-colors group"
                >
                  <td className="py-5 px-6">
                    <div className="space-y-1">
                      <p className="text-sm font-medium text-slate-200">{transaction.id}</p>
                      <p className="text-xs text-slate-500 font-mono">{transaction.tx_hash}</p>
                    </div>
                  </td>
                  <td className="py-5 px-6 text-sm text-slate-400">{transaction.date}</td>
                  <td className="py-5 px-6">
                    <p className="text-sm text-slate-200 capitalize">{transaction.title}</p>
                    <p className="text-xs text-slate-500">Backend ledger record</p>
                  </td>
                  <td className="py-5 px-6">
                    <div className="flex items-center gap-1.5">
                      {transaction.type.toLowerCase().includes("payment") ? (
                        <ArrowUpRight className="w-4 h-4 text-amber-500" />
                      ) : (
                        <ArrowDownRight className="w-4 h-4 text-emerald-500" />
                      )}
                      <span className="text-sm font-bold text-white">{formatAmount(transaction.amount, transaction.asset)}</span>
                    </div>
                  </td>
                  <td className="py-5 px-6 text-sm text-slate-400">{transaction.asset}</td>
                  <td className="py-5 px-6">
                    <span className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold border ${
                      transaction.status === "confirmed"
                        ? "bg-emerald-500/10 text-emerald-500 border-emerald-500/20"
                        : transaction.status === "awaiting"
                          ? "bg-amber-500/10 text-amber-500 border-amber-500/20"
                          : transaction.status === "initialized"
                            ? "bg-sky-500/10 text-sky-400 border-sky-500/20"
                            : "bg-rose-500/10 text-rose-500 border-rose-500/20"
                    }`}>
                      {transaction.status}
                    </span>
                  </td>
                  <td className="py-5 px-6">
                    <Link to={`/orders/${transaction.id}`} className="inline-flex items-center gap-2 text-sm text-amber-500 hover:text-amber-400">
                      Open tracker <ExternalLink className="w-3 h-3" />
                    </Link>
                  </td>
                </motion.tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
