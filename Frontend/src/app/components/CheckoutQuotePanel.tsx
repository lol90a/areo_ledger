/**
 * The priced, wallet-aware part of checkout.
 *
 * It owns the three things the security review asked about:
 *
 *  1. **Where the price comes from.** Every figure on screen is read from a
 *     server-issued quote. Nothing here multiplies, converts or rounds a price.
 *  2. **What happens when the price moves.** Re-quoting surfaces `price_changed`,
 *     the old and new totals are shown side by side, and the user must approve the
 *     new one. Acceptance echoes the exact figures back, so a stale screen is
 *     rejected by the server rather than silently charged.
 *  3. **Whether the customer can actually pay.** The connected wallet's balance is
 *     read in the token's own base units and compared with the quoted amount,
 *     before anything is signed, and re-read immediately before approval.
 */
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  AlertTriangle,
  CheckCircle2,
  Copy,
  Link2,
  Loader2,
  RefreshCw,
  ShieldCheck,
  Timer,
  Wallet,
} from "lucide-react";

import {
  acceptQuote,
  createQuote,
  initPayment,
  type InitPaymentResponse,
  type PaymentOption,
  type QuoteResponse,
} from "../lib/backend";
import {
  checkBalance,
  connectWallet,
  formatUnits,
  parseBaseUnits,
  readEvmChainId,
  shortenAddress,
  switchEvmChain,
  walletAvailable,
  watchWallet,
  type BalanceCheck,
  type ConnectedWallet,
  type PaymentRoute,
} from "../lib/wallet";

type Props = {
  token: string;
  bookingId: string | null;
  option: PaymentOption;
  onReady: (payment: InitPaymentResponse) => void;
  onEnsureBooking: () => Promise<string | null>;
};

function money(cents: number, currency: string): string {
  return `${(cents / 100).toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })} ${currency}`;
}

function routeFrom(option: PaymentOption): PaymentRoute {
  return {
    method: option.method,
    token: option.token,
    chain: option.chain,
    walletKind: option.wallet_kind,
    chainId: option.chain_id,
    tokenContract: option.token_contract,
    decimals: option.decimals,
  };
}

export function CheckoutQuotePanel({ token, bookingId, option, onReady, onEnsureBooking }: Props) {
  const route = useMemo(() => routeFrom(option), [option]);

  const [quote, setQuote] = useState<QuoteResponse | null>(null);
  const [payment, setPayment] = useState<InitPaymentResponse | null>(null);
  const [wallet, setWallet] = useState<ConnectedWallet | null>(null);
  const [balance, setBalance] = useState<BalanceCheck | null>(null);
  const [secondsLeft, setSecondsLeft] = useState(0);
  const [busy, setBusy] = useState<"quote" | "accept" | "wallet" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const expired = Boolean(quote) && secondsLeft <= 0 && quote?.status === "active";
  const locked = quote?.status === "accepted";

  /* ---------------------------------------------------------------- */
  /* Reset whenever the route changes: a quote is bound to one token   */
  /* on one chain, and so is a balance reading.                        */
  /* ---------------------------------------------------------------- */
  const methodRef = useRef(option.method);
  useEffect(() => {
    if (methodRef.current === option.method) return;
    methodRef.current = option.method;
    setQuote(null);
    setPayment(null);
    setBalance(null);
    setError(null);
  }, [option.method]);

  /* ---------------------------------------------------------------- */
  /* Quote countdown                                                   */
  /* ---------------------------------------------------------------- */
  useEffect(() => {
    if (!quote || quote.status !== "active") {
      setSecondsLeft(0);
      return;
    }
    const expiresAt = new Date(quote.expires_at).getTime();
    const tick = () => setSecondsLeft(Math.max(0, Math.floor((expiresAt - Date.now()) / 1000)));
    tick();
    const id = window.setInterval(tick, 1000);
    return () => window.clearInterval(id);
  }, [quote]);

  /* ---------------------------------------------------------------- */
  /* Balance                                                           */
  /* ---------------------------------------------------------------- */
  const refreshBalance = useCallback(
    async (current: ConnectedWallet | null, q: QuoteResponse | null) => {
      if (!current || !q) {
        setBalance(null);
        return null;
      }
      const required = parseBaseUnits(q.crypto_base_units);
      if (required === null) {
        setBalance({ state: "error", reason: "The server returned an unreadable amount." });
        return null;
      }
      // Re-read the chain id rather than trusting the value captured at connect
      // time: the user may have switched networks since.
      const live =
        current.kind === "evm" ? { ...current, chainId: await readEvmChainId() } : current;
      setWallet(live);
      const result = await checkBalance(route, live, required);
      setBalance(result);
      return result;
    },
    [route],
  );

  useEffect(() => {
    void refreshBalance(wallet, quote);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [wallet?.address, wallet?.chainId, quote?.crypto_base_units]);

  // An account or network change mid-checkout must invalidate the reading.
  useEffect(() => {
    if (!wallet) return;
    return watchWallet(wallet.kind, () => {
      void refreshBalance(wallet, quote);
    });
  }, [wallet, quote, refreshBalance]);

  /* ---------------------------------------------------------------- */
  /* Actions                                                           */
  /* ---------------------------------------------------------------- */

  const handleQuote = async () => {
    setError(null);
    setBusy("quote");
    try {
      if (!option.available) {
        throw new Error(
          `${option.token} is not currently accepted: the platform has no receiving wallet configured for this route.`,
        );
      }
      const id = bookingId ?? (await onEnsureBooking());
      if (!id) return;
      const next = await createQuote(token, { booking_id: id, method: option.method });
      setQuote(next);
      setPayment(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not get a price.");
    } finally {
      setBusy(null);
    }
  };

  const handleConnect = async () => {
    setError(null);
    setBusy("wallet");
    try {
      const connected = await connectWallet(route.walletKind);
      setWallet(connected);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not connect the wallet.");
    } finally {
      setBusy(null);
    }
  };

  const handleSwitchNetwork = async () => {
    if (route.chainId === null) return;
    setError(null);
    setBusy("wallet");
    try {
      await switchEvmChain(route.chainId);
      await refreshBalance(wallet, quote);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not switch network.");
    } finally {
      setBusy(null);
    }
  };

  const handleAccept = async () => {
    if (!quote || !bookingId) return;
    setError(null);
    setBusy("accept");
    try {
      // Balances move. Re-read immediately before locking the price rather than
      // relying on whatever was on screen.
      if (wallet) {
        const fresh = await refreshBalance(wallet, quote);
        if (fresh?.state === "ok" && !fresh.sufficient) {
          throw new Error(
            `Not enough ${option.token}: you need ${formatUnits(fresh.required, option.decimals)} and hold ${formatUnits(fresh.balance, option.decimals)}.`,
          );
        }
      }

      const accepted = await acceptQuote(token, {
        booking_id: bookingId,
        quote_id: quote.quote_id,
        approved_total_cents: quote.total_cents,
        approved_crypto_base_units: quote.crypto_base_units,
      });
      setQuote(accepted);

      const init = await initPayment(token, { booking_id: bookingId });
      setPayment(init);
      onReady(init);
    } catch (err) {
      // A 409 here is the price-change path: fetch the new quote so the user sees
      // what actually changed instead of a bare error.
      const message = err instanceof Error ? err.message : "Could not approve the price.";
      setError(message);
      if (bookingId) {
        try {
          setQuote(await createQuote(token, { booking_id: bookingId, method: option.method }));
        } catch {
          /* keep the original error on screen */
        }
      }
    } finally {
      setBusy(null);
    }
  };

  const handleCopy = async () => {
    const address = payment?.wallet_address ?? quote?.receiver_address;
    if (!address) return;
    await navigator.clipboard.writeText(address);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1800);
  };

  /* ---------------------------------------------------------------- */
  /* Derived UI state                                                  */
  /* ---------------------------------------------------------------- */

  const wrongNetwork =
    route.walletKind === "evm" &&
    wallet !== null &&
    route.chainId !== null &&
    wallet.chainId !== route.chainId;

  const insufficient = balance?.state === "ok" && !balance.sufficient;

  const canApprove =
    Boolean(quote) &&
    quote?.status === "active" &&
    !expired &&
    !wrongNetwork &&
    !insufficient &&
    busy === null;

  const minutes = Math.floor(secondsLeft / 60);
  const seconds = String(secondsLeft % 60).padStart(2, "0");

  return (
    <div className="space-y-6">
      {error && (
        <div className="rounded-2xl border border-rose-500/25 bg-rose-500/10 px-4 py-3 text-sm text-rose-200">
          {error}
        </div>
      )}

      {/* ---------------------------------------------------------- Price */}
      <div className="rounded-3xl border border-white/8 bg-white/[0.03] p-6">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Server-authoritative price</p>
          <button
            type="button"
            onClick={handleQuote}
            disabled={busy !== null || locked}
            className="inline-flex items-center gap-2 rounded-2xl border border-white/10 px-4 py-2 text-sm text-white hover:bg-white/[0.05] disabled:opacity-40"
          >
            {busy === "quote" ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <RefreshCw className="h-4 w-4 text-amber-400" />
            )}
            {quote ? "Refresh price" : "Get price"}
          </button>
        </div>

        {!quote && (
          <p className="mt-4 text-sm text-slate-400">
            The amount is calculated by the backend from the live catalogue price and an
            exchange rate it records. Request a quote to see the exact figure and how long it
            is held for.
          </p>
        )}

        {quote?.price_changed && quote.previous_total_cents !== null && (
          <div className="mt-4 rounded-2xl border border-amber-500/30 bg-amber-500/10 p-4">
            <div className="flex gap-3">
              <AlertTriangle className="mt-0.5 h-5 w-5 shrink-0 text-amber-400" />
              <div className="text-sm text-amber-100/90">
                <p className="font-semibold text-amber-200">The price changed.</p>
                <p className="mt-1">
                  This item was{" "}
                  <span className="line-through">
                    {money(quote.previous_total_cents, quote.currency)}
                  </span>{" "}
                  and is now{" "}
                  <span className="font-semibold text-white">
                    {money(quote.total_cents, quote.currency)}
                  </span>
                  . Nothing is charged until you approve the new amount.
                </p>
              </div>
            </div>
          </div>
        )}

        {quote && (
          <div className="mt-5 space-y-4">
            <div className="rounded-2xl border border-white/8 bg-black/30 p-5">
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-slate-500">Base price</span>
                  <span className="text-white">{money(quote.base_cents, quote.currency)}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-500">Markup</span>
                  <span className="text-white">{money(quote.markup_cents, quote.currency)}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-500">Service fee</span>
                  <span className="text-white">{money(quote.service_fee_cents, quote.currency)}</span>
                </div>
                <div className="flex justify-between border-t border-white/10 pt-3 text-base">
                  <span className="font-medium text-slate-300">Total</span>
                  <span className="font-bold text-white">
                    {money(quote.total_cents, quote.currency)}
                  </span>
                </div>
              </div>

              <div className="mt-5 border-t border-white/10 pt-4">
                <p className="text-2xl font-bold text-amber-400">
                  Send exactly {quote.crypto_amount} {quote.token}
                </p>
                <p className="mt-1 font-mono text-xs text-slate-500">
                  {quote.crypto_base_units} base units on {quote.chain}
                </p>
                <p className="mt-2 text-xs text-slate-500">
                  Rate {quote.unit_price_usd} {quote.currency}/{quote.token} from{" "}
                  <span className="text-slate-400">{quote.rate_source}</span>, observed{" "}
                  {new Date(quote.rate_observed_at).toLocaleTimeString()}
                </p>
              </div>
            </div>

            <div
              className={`flex items-center gap-2 rounded-2xl border px-4 py-3 text-sm ${
                locked
                  ? "border-emerald-500/25 bg-emerald-500/10 text-emerald-200"
                  : expired
                    ? "border-rose-500/25 bg-rose-500/10 text-rose-200"
                    : "border-white/10 bg-white/[0.03] text-slate-300"
              }`}
            >
              {locked ? (
                <>
                  <ShieldCheck className="h-4 w-4" />
                  Price locked. This amount will not change.
                </>
              ) : expired ? (
                <>
                  <Timer className="h-4 w-4" />
                  This quote expired. Refresh the price to continue.
                </>
              ) : (
                <>
                  <Timer className="h-4 w-4 text-amber-400" />
                  Held for {minutes}:{seconds}
                </>
              )}
            </div>
          </div>
        )}
      </div>

      {/* --------------------------------------------------------- Wallet */}
      <div className="rounded-3xl border border-white/8 bg-white/[0.03] p-6">
        <p className="text-xs uppercase tracking-[0.24em] text-slate-500">Wallet</p>

        {route.walletKind === "manual" ? (
          <p className="mt-4 text-sm text-slate-400">
            {option.token} has no browser wallet standard, so the balance cannot be checked
            here. Send the exact amount shown above from your own wallet and submit the
            transaction id.
          </p>
        ) : !walletAvailable(route.walletKind) ? (
          <p className="mt-4 text-sm text-amber-200/90">
            No {route.walletKind === "evm" ? "EVM" : "Solana"} wallet detected in this browser.
            You can still pay manually by sending the exact amount to the address below.
          </p>
        ) : !wallet ? (
          <button
            type="button"
            onClick={handleConnect}
            disabled={busy !== null}
            className="mt-4 inline-flex items-center gap-2 rounded-2xl bg-white/[0.06] px-5 py-3 text-sm font-medium text-white hover:bg-white/[0.1] disabled:opacity-40"
          >
            {busy === "wallet" ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Wallet className="h-4 w-4 text-amber-400" />
            )}
            Connect wallet
          </button>
        ) : (
          <div className="mt-4 space-y-4">
            <div className="flex flex-wrap items-center gap-3 text-sm">
              <span className="inline-flex items-center gap-2 rounded-full border border-emerald-500/25 bg-emerald-500/10 px-3 py-1.5 text-emerald-200">
                <Link2 className="h-3.5 w-3.5" />
                {shortenAddress(wallet.address)}
              </span>
              {wallet.chainId !== null && (
                <span className="text-xs text-slate-500">chain id {wallet.chainId}</span>
              )}
            </div>

            {wrongNetwork && (
              <div className="rounded-2xl border border-amber-500/30 bg-amber-500/10 p-4 text-sm text-amber-100/90">
                <p className="font-semibold text-amber-200">Wrong network.</p>
                <p className="mt-1">
                  This payment settles on chain {route.chainId}; your wallet is on{" "}
                  {wallet.chainId ?? "an unknown chain"}. Sending from the wrong network loses
                  the funds.
                </p>
                <button
                  type="button"
                  onClick={handleSwitchNetwork}
                  disabled={busy !== null}
                  className="mt-3 rounded-xl border border-amber-400/40 px-4 py-2 text-sm text-amber-100 hover:bg-amber-500/15 disabled:opacity-40"
                >
                  Switch network
                </button>
              </div>
            )}

            {balance?.state === "ok" && (
              <div
                className={`rounded-2xl border p-4 text-sm ${
                  balance.sufficient
                    ? "border-white/10 bg-black/30 text-slate-300"
                    : "border-rose-500/30 bg-rose-500/10 text-rose-100"
                }`}
              >
                <div className="flex justify-between">
                  <span className="text-slate-400">Available</span>
                  <span className="font-mono text-white">
                    {formatUnits(balance.balance, option.decimals)} {option.token}
                  </span>
                </div>
                <div className="mt-1 flex justify-between">
                  <span className="text-slate-400">Required</span>
                  <span className="font-mono text-white">
                    {formatUnits(balance.required, option.decimals)} {option.token}
                  </span>
                </div>
                {!balance.sufficient && (
                  <p className="mt-3 font-medium">
                    Short by {formatUnits(balance.shortfall, option.decimals)} {option.token}{" "}
                    (including an allowance for network fees). Top up or choose another
                    network.
                  </p>
                )}
                {balance.sufficient && balance.gasLooksThin && (
                  <p className="mt-3 text-amber-200">
                    Your native balance is low; the transaction may fail on fees.
                  </p>
                )}
              </div>
            )}

            {balance?.state === "error" && (
              <p className="text-sm text-amber-200/90">{balance.reason}</p>
            )}
            {balance?.state === "unsupported" && (
              <p className="text-sm text-slate-400">{balance.reason}</p>
            )}

            <p className="text-xs leading-5 text-slate-600">
              This balance check is a convenience only. What you owe is fixed by the quote
              above, and the backend verifies the actual on-chain transfer before any order is
              confirmed.
            </p>
          </div>
        )}
      </div>

      {/* --------------------------------------------------------- Approve */}
      {!locked ? (
        <button
          type="button"
          onClick={handleAccept}
          disabled={!canApprove}
          className="w-full rounded-2xl bg-amber-500 px-6 py-4 font-semibold text-black transition-colors hover:bg-amber-400 disabled:cursor-not-allowed disabled:opacity-40"
        >
          {busy === "accept" ? (
            <span className="inline-flex items-center gap-2">
              <Loader2 className="h-4 w-4 animate-spin" /> Locking price...
            </span>
          ) : !quote ? (
            "Get a price first"
          ) : expired ? (
            "Quote expired - refresh the price"
          ) : wrongNetwork ? (
            "Switch to the correct network"
          ) : insufficient ? (
            `Insufficient ${option.token} balance`
          ) : (
            `Approve ${money(quote.total_cents, quote.currency)} and lock the price`
          )}
        </button>
      ) : (
        <div className="rounded-3xl border border-emerald-500/25 bg-emerald-500/10 p-6">
          <div className="flex items-center gap-2 text-emerald-200">
            <CheckCircle2 className="h-5 w-5" />
            <p className="font-semibold">Price locked - send the payment</p>
          </div>
          <p className="mt-4 text-2xl font-bold text-white">
            {payment?.crypto_amount ?? quote?.crypto_amount} {option.token}
          </p>
          <p className="mt-3 text-xs uppercase tracking-[0.24em] text-slate-500">
            To this address on {option.chain}
          </p>
          <p className="mt-2 break-all font-mono text-sm text-slate-300">
            {payment?.wallet_address ?? quote?.receiver_address}
          </p>
          <button
            type="button"
            onClick={handleCopy}
            className="mt-4 inline-flex items-center gap-2 rounded-2xl border border-white/10 px-4 py-3 text-sm text-white hover:bg-white/[0.05]"
          >
            {copied ? (
              <CheckCircle2 className="h-4 w-4 text-emerald-400" />
            ) : (
              <Copy className="h-4 w-4 text-amber-500" />
            )}
            Copy address
          </button>
          <p className="mt-4 text-xs leading-5 text-slate-500">
            Send from the wallet you control, on {option.chain} only. Then submit the
            transaction id below; the order is confirmed when the backend has verified the
            transfer on-chain, not before.
          </p>
        </div>
      )}
    </div>
  );
}
