import { type FormEvent, useEffect, useState } from "react";
import { motion } from "motion/react";
import { Link, useNavigate } from "react-router";
import {
  ArrowRight,
  Eye,
  EyeOff,
  LockKeyhole,
  Mail,
  ShieldCheck,
  Sparkles,
  Wallet,
} from "lucide-react";

import { Input } from "../components/ui/input";
import { Button } from "../components/ui/button";
import { login } from "../lib/backend";
import { getAuthSession, setAuthSession } from "../lib/auth";

const trustSignals = [
  "Institutional-grade payment verification",
  "Priority handling for charter and asset purchases",
  "Protected transaction history and concierge access",
];

const statusItems = [
  { label: "Supported assets", value: "BTC, ETH, USDT, USDC, SOL, BNB" },
  { label: "Average verification", value: "Under 3 minutes" },
  { label: "Client support", value: "24/7 global concierge desk" },
];

export default function Signin() {
  const navigate = useNavigate();
  const [showPassword, setShowPassword] = useState(false);
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (getAuthSession()) {
      navigate("/transactions", { replace: true });
    }
  }, [navigate]);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setError(null);
    setLoading(true);

    try {
      const session = await login({
        email: email.trim(),
        password,
      });
      setAuthSession(session);
      navigate("/transactions", { replace: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unable to sign in right now.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="relative overflow-hidden">
      <div className="absolute inset-0 bg-[#05050A]" />
      <div className="absolute left-1/2 top-0 h-[34rem] w-[34rem] -translate-x-1/2 rounded-full bg-amber-500/10 blur-3xl" />
      <div className="absolute -left-20 top-40 h-72 w-72 rounded-full bg-sky-500/10 blur-3xl" />
      <div className="absolute -right-16 bottom-10 h-80 w-80 rounded-full bg-amber-300/10 blur-3xl" />

      <section className="relative max-w-7xl mx-auto px-6 py-8 md:py-12">
        <div className="grid grid-cols-1 gap-8 lg:grid-cols-[1.1fr_0.9fr] lg:items-stretch">
          <motion.div
            initial={{ opacity: 0, y: 24 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.7, ease: "easeOut" }}
            className="relative overflow-hidden rounded-[2rem] border border-white/8 bg-gradient-to-br from-white/8 via-white/4 to-transparent p-8 shadow-[0_35px_120px_rgba(0,0,0,0.45)] backdrop-blur-xl md:p-10"
          >
            <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_left,rgba(245,158,11,0.16),transparent_36%),radial-gradient(circle_at_bottom_right,rgba(56,189,248,0.14),transparent_28%)]" />

            <div className="relative">
              <div className="mb-8 inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/6 px-4 py-2 text-xs font-medium uppercase tracking-[0.28em] text-slate-300 backdrop-blur-md">
                <Sparkles className="h-3.5 w-3.5 text-amber-400" />
                Secure Client Access
              </div>

              <h1 className="max-w-xl text-4xl font-bold tracking-tight text-white md:text-5xl lg:text-6xl">
                Sign in to your
                <span className="block bg-gradient-to-r from-amber-200 via-amber-400 to-amber-600 bg-clip-text text-transparent">
                  private terminal
                </span>
              </h1>

              <p className="mt-6 max-w-2xl text-base leading-7 text-slate-400 md:text-lg">
                Access your bookings, portfolio activity, payment tracking, and white-glove support from one protected dashboard.
              </p>

              <div className="mt-10 grid gap-4 sm:grid-cols-3">
                {statusItems.map((item) => (
                  <div
                    key={item.label}
                    className="rounded-2xl border border-white/8 bg-black/30 p-4 backdrop-blur-md"
                  >
                    <p className="text-[11px] uppercase tracking-[0.24em] text-slate-500">
                      {item.label}
                    </p>
                    <p className="mt-3 text-sm font-medium leading-6 text-white">
                      {item.value}
                    </p>
                  </div>
                ))}
              </div>

              <div className="mt-10 rounded-[1.75rem] border border-white/8 bg-black/30 p-6 backdrop-blur-md">
                <div className="flex items-center justify-between gap-4 border-b border-white/8 pb-5">
                  <div>
                    <p className="text-xs uppercase tracking-[0.28em] text-amber-500">
                      Session Snapshot
                    </p>
                    <h2 className="mt-2 text-2xl font-semibold text-white">
                      Premium account access
                    </h2>
                  </div>
                  <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-amber-500/12">
                    <Wallet className="h-6 w-6 text-amber-400" />
                  </div>
                </div>

                <div className="mt-6 space-y-4">
                  {trustSignals.map((item) => (
                    <div
                      key={item}
                      className="flex items-start gap-3 rounded-2xl border border-white/6 bg-white/[0.03] px-4 py-4"
                    >
                      <ShieldCheck className="mt-0.5 h-5 w-5 text-emerald-400" />
                      <p className="text-sm leading-6 text-slate-300">{item}</p>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 24 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.7, delay: 0.08, ease: "easeOut" }}
            className="relative overflow-hidden rounded-[2rem] border border-white/8 bg-[#0A0A0E]/95 p-7 shadow-[0_35px_120px_rgba(0,0,0,0.55)] backdrop-blur-xl md:p-9"
          >
            <div className="absolute inset-x-10 top-0 h-px bg-gradient-to-r from-transparent via-amber-500/60 to-transparent" />

            <div className="relative">
              <div className="mb-8 flex items-center justify-between gap-4">
                <div>
                  <p className="text-xs uppercase tracking-[0.28em] text-slate-500">
                    Account Access
                  </p>
                  <h2 className="mt-2 text-3xl font-bold text-white">Welcome back</h2>
                </div>
                <div className="flex h-12 w-12 items-center justify-center rounded-2xl border border-amber-500/20 bg-amber-500/10">
                  <LockKeyhole className="h-5 w-5 text-amber-400" />
                </div>
              </div>

              <form className="space-y-5" onSubmit={handleSubmit}>
                <div className="space-y-2">
                  <label
                    htmlFor="email"
                    className="text-xs font-semibold uppercase tracking-[0.24em] text-slate-500"
                  >
                    Email address
                  </label>
                  <div className="relative">
                    <Mail className="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
                    <Input
                      id="email"
                      type="email"
                      value={email}
                      onChange={(event) => setEmail(event.target.value)}
                      placeholder="client@aeroledger.com"
                      className="h-13 rounded-2xl border-white/10 bg-white/[0.04] pl-11 text-white placeholder:text-slate-500 focus-visible:border-amber-500/60 focus-visible:ring-amber-500/20"
                    />
                  </div>
                </div>

                <div className="space-y-2">
                  <div className="flex items-center justify-between gap-4">
                    <label
                      htmlFor="password"
                      className="text-xs font-semibold uppercase tracking-[0.24em] text-slate-500"
                    >
                      Password
                    </label>
                    <button
                      type="button"
                      className="text-xs font-medium text-amber-500 transition-colors hover:text-amber-400"
                    >
                      Forgot password?
                    </button>
                  </div>

                  <div className="relative">
                    <LockKeyhole className="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
                    <Input
                      id="password"
                      type={showPassword ? "text" : "password"}
                      value={password}
                      onChange={(event) => setPassword(event.target.value)}
                      placeholder="Enter your password"
                      className="h-13 rounded-2xl border-white/10 bg-white/[0.04] pl-11 pr-12 text-white placeholder:text-slate-500 focus-visible:border-amber-500/60 focus-visible:ring-amber-500/20"
                    />
                    <button
                      type="button"
                      onClick={() => setShowPassword((current) => !current)}
                      className="absolute right-4 top-1/2 -translate-y-1/2 text-slate-500 transition-colors hover:text-slate-300"
                      aria-label={showPassword ? "Hide password" : "Show password"}
                    >
                      {showPassword ? (
                        <EyeOff className="h-4 w-4" />
                      ) : (
                        <Eye className="h-4 w-4" />
                      )}
                    </button>
                  </div>
                </div>

                <div className="flex flex-col gap-3 rounded-2xl border border-white/8 bg-white/[0.03] px-4 py-4 sm:flex-row sm:items-center sm:justify-between">
                  <label className="flex items-center gap-3 text-sm text-slate-300">
                    <input
                      type="checkbox"
                      defaultChecked
                      className="h-4 w-4 rounded border border-white/15 bg-transparent accent-amber-500"
                    />
                    Keep this terminal trusted for 30 days
                  </label>
                  <span className="text-xs uppercase tracking-[0.24em] text-slate-500">
                    Biometric ready
                  </span>
                </div>

                {error && (
                  <div className="rounded-2xl border border-rose-500/20 bg-rose-500/10 px-4 py-3 text-sm text-rose-200">
                    {error}
                  </div>
                )}

                <Button
                  type="submit"
                  disabled={loading}
                  className="h-13 w-full rounded-2xl bg-amber-500 text-base font-semibold text-black shadow-[0_0_24px_rgba(245,158,11,0.25)] transition-all hover:bg-amber-400 hover:shadow-[0_0_34px_rgba(245,158,11,0.38)] disabled:opacity-70"
                >
                  {loading ? "Signing In..." : "Sign In"}
                  <ArrowRight className="h-4 w-4" />
                </Button>
              </form>

              <div className="my-7 flex items-center gap-4">
                <div className="h-px flex-1 bg-white/10" />
                <span className="text-xs uppercase tracking-[0.24em] text-slate-500">
                  or continue
                </span>
                <div className="h-px flex-1 bg-white/10" />
              </div>

              <div className="grid gap-3 sm:grid-cols-2">
                <button
                  type="button"
                  className="rounded-2xl border border-white/10 bg-white/[0.03] px-4 py-4 text-sm font-medium text-white transition-all hover:border-amber-500/30 hover:bg-white/[0.06]"
                >
                  Connect Wallet
                </button>
                <button
                  type="button"
                  className="rounded-2xl border border-white/10 bg-white/[0.03] px-4 py-4 text-sm font-medium text-white transition-all hover:border-amber-500/30 hover:bg-white/[0.06]"
                >
                  Request Concierge Access
                </button>
              </div>

              <div className="mt-8 rounded-[1.75rem] border border-white/8 bg-gradient-to-br from-white/[0.04] to-transparent p-5">
                <p className="text-xs uppercase tracking-[0.24em] text-slate-500">
                  New to AeroLedger?
                </p>
                <p className="mt-3 text-sm leading-6 text-slate-300">
                  Create a client profile to manage flight manifests, aircraft inquiries, and crypto settlement history.
                </p>
                <Link
                  to="/signup"
                  className="mt-4 inline-flex items-center gap-2 text-sm font-medium text-amber-500 transition-colors hover:text-amber-400"
                >
                  Create an account
                  <ArrowRight className="h-4 w-4" />
                </Link>
              </div>
            </div>
          </motion.div>
        </div>
      </section>
    </div>
  );
}

