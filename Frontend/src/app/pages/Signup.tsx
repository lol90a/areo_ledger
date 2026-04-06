import { type FormEvent, useEffect, useState } from "react";
import { motion } from "motion/react";
import { Link, useNavigate } from "react-router";
import {
  ArrowRight,
  BadgeCheck,
  Eye,
  EyeOff,
  LockKeyhole,
  Mail,
  ShieldCheck,
  Sparkles,
  UserRound,
  Wallet,
} from "lucide-react";

import { Input } from "../components/ui/input";
import { Button } from "../components/ui/button";
import { signup } from "../lib/backend";
import { getAuthSession, setAuthSession } from "../lib/auth";

const onboardingSteps = [
  "Create your secure client identity",
  "Set preferred payment and concierge access",
  "Manage bookings, assets, and settlement tracking",
];

const membershipSignals = [
  { label: "Onboarding time", value: "Under 2 minutes" },
  { label: "Account tier", value: "Private client + operator ready" },
  { label: "Coverage", value: "Flights, marketplace, payments, tracking" },
];

export default function Signup() {
  const navigate = useNavigate();
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [firstName, setFirstName] = useState("");
  const [lastName, setLastName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [acceptedTerms, setAcceptedTerms] = useState(true);
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

    if (!acceptedTerms) {
      setError("Please accept the account and verification terms.");
      return;
    }

    if (password !== confirmPassword) {
      setError("Passwords do not match.");
      return;
    }

    const name = `${firstName.trim()} ${lastName.trim()}`.trim();
    if (!name) {
      setError("Please provide your full name.");
      return;
    }

    setLoading(true);
    try {
      const session = await signup({
        email: email.trim(),
        name,
        password,
      });
      setAuthSession(session);
      navigate("/transactions", { replace: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unable to create your account right now.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="relative overflow-hidden">
      <div className="absolute inset-0 bg-[#05050A]" />
      <div className="absolute left-1/2 top-0 h-[34rem] w-[34rem] -translate-x-1/2 rounded-full bg-amber-500/10 blur-3xl" />
      <div className="absolute -left-20 top-56 h-72 w-72 rounded-full bg-cyan-500/10 blur-3xl" />
      <div className="absolute -right-20 bottom-0 h-96 w-96 rounded-full bg-amber-300/10 blur-3xl" />

      <section className="relative mx-auto max-w-7xl px-6 py-8 md:py-12">
        <div className="grid grid-cols-1 gap-8 lg:grid-cols-[0.95fr_1.05fr] lg:items-stretch">
          <motion.div
            initial={{ opacity: 0, y: 24 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.7, ease: "easeOut" }}
            className="relative overflow-hidden rounded-[2rem] border border-white/8 bg-[#0A0A0E]/95 p-7 shadow-[0_35px_120px_rgba(0,0,0,0.55)] backdrop-blur-xl md:p-9"
          >
            <div className="absolute inset-x-10 top-0 h-px bg-gradient-to-r from-transparent via-amber-500/60 to-transparent" />

            <div className="relative">
              <div className="mb-8 flex items-center justify-between gap-4">
                <div>
                  <p className="text-xs uppercase tracking-[0.28em] text-slate-500">
                    New Account
                  </p>
                  <h1 className="mt-2 text-3xl font-bold text-white md:text-4xl">
                    Create your AeroLedger profile
                  </h1>
                </div>
                <div className="flex h-12 w-12 items-center justify-center rounded-2xl border border-amber-500/20 bg-amber-500/10">
                  <BadgeCheck className="h-5 w-5 text-amber-400" />
                </div>
              </div>

              <p className="mb-8 max-w-xl text-sm leading-6 text-slate-400 md:text-base">
                Open a premium account to coordinate bookings, review asset opportunities, and track high-value crypto settlements from a single secure workspace.
              </p>

              <form className="space-y-5" onSubmit={handleSubmit}>
                <div className="grid gap-5 sm:grid-cols-2">
                  <div className="space-y-2">
                    <label htmlFor="firstName" className="text-xs font-semibold uppercase tracking-[0.24em] text-slate-500">
                      First name
                    </label>
                    <div className="relative">
                      <UserRound className="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
                      <Input
                        id="firstName"
                        type="text"
                        value={firstName}
                        onChange={(event) => setFirstName(event.target.value)}
                        placeholder="Ali"
                        className="h-13 rounded-2xl border-white/10 bg-white/[0.04] pl-11 text-white placeholder:text-slate-500 focus-visible:border-amber-500/60 focus-visible:ring-amber-500/20"
                      />
                    </div>
                  </div>

                  <div className="space-y-2">
                    <label htmlFor="lastName" className="text-xs font-semibold uppercase tracking-[0.24em] text-slate-500">
                      Last name
                    </label>
                    <Input
                      id="lastName"
                      type="text"
                      value={lastName}
                      onChange={(event) => setLastName(event.target.value)}
                      placeholder="Hassan"
                      className="h-13 rounded-2xl border-white/10 bg-white/[0.04] text-white placeholder:text-slate-500 focus-visible:border-amber-500/60 focus-visible:ring-amber-500/20"
                    />
                  </div>
                </div>

                <div className="space-y-2">
                  <label htmlFor="email" className="text-xs font-semibold uppercase tracking-[0.24em] text-slate-500">
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

                <div className="grid gap-5 sm:grid-cols-2">
                  <div className="space-y-2">
                    <label htmlFor="password" className="text-xs font-semibold uppercase tracking-[0.24em] text-slate-500">
                      Password
                    </label>
                    <div className="relative">
                      <LockKeyhole className="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
                      <Input
                        id="password"
                        type={showPassword ? "text" : "password"}
                        value={password}
                        onChange={(event) => setPassword(event.target.value)}
                        placeholder="Create password"
                        className="h-13 rounded-2xl border-white/10 bg-white/[0.04] pl-11 pr-12 text-white placeholder:text-slate-500 focus-visible:border-amber-500/60 focus-visible:ring-amber-500/20"
                      />
                      <button
                        type="button"
                        onClick={() => setShowPassword((current) => !current)}
                        className="absolute right-4 top-1/2 -translate-y-1/2 text-slate-500 transition-colors hover:text-slate-300"
                        aria-label={showPassword ? "Hide password" : "Show password"}
                      >
                        {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                      </button>
                    </div>
                  </div>

                  <div className="space-y-2">
                    <label htmlFor="confirmPassword" className="text-xs font-semibold uppercase tracking-[0.24em] text-slate-500">
                      Confirm password
                    </label>
                    <div className="relative">
                      <LockKeyhole className="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-500" />
                      <Input
                        id="confirmPassword"
                        type={showConfirmPassword ? "text" : "password"}
                        value={confirmPassword}
                        onChange={(event) => setConfirmPassword(event.target.value)}
                        placeholder="Repeat password"
                        className="h-13 rounded-2xl border-white/10 bg-white/[0.04] pl-11 pr-12 text-white placeholder:text-slate-500 focus-visible:border-amber-500/60 focus-visible:ring-amber-500/20"
                      />
                      <button
                        type="button"
                        onClick={() => setShowConfirmPassword((current) => !current)}
                        className="absolute right-4 top-1/2 -translate-y-1/2 text-slate-500 transition-colors hover:text-slate-300"
                        aria-label={showConfirmPassword ? "Hide password" : "Show password"}
                      >
                        {showConfirmPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                      </button>
                    </div>
                  </div>
                </div>

                <div className="rounded-2xl border border-white/8 bg-white/[0.03] px-4 py-4">
                  <label className="flex items-start gap-3 text-sm text-slate-300">
                    <input
                      type="checkbox"
                      checked={acceptedTerms}
                      onChange={(event) => setAcceptedTerms(event.target.checked)}
                      className="mt-1 h-4 w-4 rounded border border-white/15 bg-transparent accent-amber-500"
                    />
                    <span className="leading-6">
                      I agree to secure account monitoring, transaction verification workflows, and the platform terms for high-value bookings and purchases.
                    </span>
                  </label>
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
                  {loading ? "Creating Account..." : "Create Account"}
                  <ArrowRight className="h-4 w-4" />
                </Button>
              </form>

              <div className="mt-8 rounded-[1.5rem] border border-white/8 bg-white/[0.03] px-5 py-4 text-sm text-slate-300">
                Already have access?
                <Link to="/signin" className="ml-2 font-medium text-amber-500 transition-colors hover:text-amber-400">
                  Sign in here
                </Link>
              </div>
            </div>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 24 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.7, delay: 0.08, ease: "easeOut" }}
            className="relative overflow-hidden rounded-[2rem] border border-white/8 bg-gradient-to-br from-white/8 via-white/4 to-transparent p-8 shadow-[0_35px_120px_rgba(0,0,0,0.45)] backdrop-blur-xl md:p-10"
          >
            <div className="absolute inset-0 bg-[radial-gradient(circle_at_top_left,rgba(245,158,11,0.16),transparent_36%),radial-gradient(circle_at_bottom_right,rgba(56,189,248,0.14),transparent_28%)]" />

            <div className="relative">
              <div className="mb-8 inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/6 px-4 py-2 text-xs font-medium uppercase tracking-[0.28em] text-slate-300 backdrop-blur-md">
                <Sparkles className="h-3.5 w-3.5 text-amber-400" />
                Premium Membership Setup
              </div>

              <h2 className="max-w-xl text-4xl font-bold tracking-tight text-white md:text-5xl lg:text-6xl">
                Join the
                <span className="block bg-gradient-to-r from-amber-200 via-amber-400 to-amber-600 bg-clip-text text-transparent">
                  private network
                </span>
              </h2>

              <p className="mt-6 max-w-2xl text-base leading-7 text-slate-400 md:text-lg">
                Designed for clients moving seamlessly between aviation, luxury assets, and crypto settlement with concierge-grade support.
              </p>

              <div className="mt-10 grid gap-4 sm:grid-cols-3">
                {membershipSignals.map((item) => (
                  <div key={item.label} className="rounded-2xl border border-white/8 bg-black/30 p-4 backdrop-blur-md">
                    <p className="text-[11px] uppercase tracking-[0.24em] text-slate-500">{item.label}</p>
                    <p className="mt-3 text-sm font-medium leading-6 text-white">{item.value}</p>
                  </div>
                ))}
              </div>

              <div className="mt-10 rounded-[1.75rem] border border-white/8 bg-black/30 p-6 backdrop-blur-md">
                <div className="flex items-center justify-between gap-4 border-b border-white/8 pb-5">
                  <div>
                    <p className="text-xs uppercase tracking-[0.28em] text-amber-500">Membership Flow</p>
                    <h3 className="mt-2 text-2xl font-semibold text-white">What unlocks next</h3>
                  </div>
                  <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-amber-500/12">
                    <Wallet className="h-6 w-6 text-amber-400" />
                  </div>
                </div>

                <div className="mt-6 space-y-4">
                  {onboardingSteps.map((item) => (
                    <div key={item} className="flex items-start gap-3 rounded-2xl border border-white/6 bg-white/[0.03] px-4 py-4">
                      <ShieldCheck className="mt-0.5 h-5 w-5 text-emerald-400" />
                      <p className="text-sm leading-6 text-slate-300">{item}</p>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </motion.div>
        </div>
      </section>
    </div>
  );
}

