/**
 * Wallet connection and pre-flight balance checks.
 *
 * ## What this is, and what it is not
 *
 * Everything here is a **UX pre-flight**. It exists so a customer is told "you are
 * 0.4 ETH short" before they sign anything, instead of discovering it from a failed
 * transaction. It is NOT a security control: a browser can report any balance it
 * likes. The amount a buyer actually owes is fixed by the server-signed quote, and
 * the only things that enforce payment are the chain itself and the backend's
 * verification of the settled transaction.
 *
 * ## Why no wallet SDK
 *
 * This talks to EIP-1193 (`window.ethereum`) and the Solana wallet-provider
 * interface (`window.solana`) directly. Both are stable, injected standards that
 * MetaMask, Rabby, Phantom, Solflare and the rest implement. Pulling in wagmi /
 * viem / `@solana/wallet-adapter` would add a large dependency tree to a React 18 +
 * Vite app for connect-and-read-balance, which is all the checkout needs. The
 * trade-off (no WalletConnect, no mobile deep-linking, no multi-wallet modal) is
 * recorded in SECURITY_AUDIT.md under FE-06.
 *
 * All amounts are handled as `bigint` in the token's smallest unit. No floats touch
 * a balance comparison.
 */

export type WalletKind = "evm" | "solana" | "manual";

/** Everything the checkout needs to know about where a payment must land. */
export type PaymentRoute = {
  method: string;
  token: string;
  chain: string;
  walletKind: WalletKind;
  /** EIP-155 chain id for EVM routes, supplied by the backend. */
  chainId: number | null;
  /** ERC-20 contract for non-native EVM tokens, supplied by the backend. */
  tokenContract: string | null;
  decimals: number;
  /**
   * RPC endpoint for chains with no chain id, supplied by the backend.
   *
   * Solana has no EIP-155 equivalent, so this used to come from the browser's
   * own `VITE_SOLANA_RPC_URL` while the backend verified against whatever
   * `SOLANA_RPC_URL` it had. The two silently disagreed -- balances read on
   * devnet, verification on mainnet -- and a payment in that state can never
   * confirm. The server owns the network for the same reason it owns chainId.
   */
  rpcUrl: string | null;
};

export type ConnectedWallet = {
  kind: Exclude<WalletKind, "manual">;
  address: string;
  /** Chain id the wallet is currently on (EVM only). */
  chainId: number | null;
};

export type BalanceCheck =
  | { state: "unsupported"; reason: string }
  | { state: "error"; reason: string }
  | {
      state: "ok";
      /** Spendable balance of the payment token, in its smallest unit. */
      balance: bigint;
      /** Required amount, in the same unit. */
      required: bigint;
      sufficient: boolean;
      /** How much more is needed; 0n when sufficient. */
      shortfall: bigint;
      /**
       * Native-coin balance, for gas. Equal to `balance` on a native route.
       * `null` when it could not be read.
       */
      nativeBalance: bigint | null;
      /** True when the native balance looks too thin to cover fees. */
      gasLooksThin: boolean;
    };

/* ------------------------------------------------------------------ */
/* Injected-provider types                                            */
/* ------------------------------------------------------------------ */

type Eip1193Provider = {
  request(args: { method: string; params?: unknown[] | object }): Promise<unknown>;
  on?(event: string, handler: (...args: unknown[]) => void): void;
  removeListener?(event: string, handler: (...args: unknown[]) => void): void;
};

type SolanaProvider = {
  isPhantom?: boolean;
  publicKey?: { toString(): string } | null;
  connect(opts?: { onlyIfTrusted?: boolean }): Promise<{ publicKey: { toString(): string } }>;
  disconnect?(): Promise<void>;
  on?(event: string, handler: (...args: unknown[]) => void): void;
  removeListener?(event: string, handler: (...args: unknown[]) => void): void;
};

declare global {
  interface Window {
    ethereum?: Eip1193Provider;
    solana?: SolanaProvider;
  }
}

/**
 * Lamports kept back for the signature fee and, where an account has to be
 * created, rent exemption. 0.003 SOL is generous for a plain transfer (5,000
 * lamports) plus one rent-exempt token account (~2,040,000 lamports).
 */
const SOLANA_FEE_BUFFER_LAMPORTS = 3_000_000n;

/** Wei kept back for gas on EVM routes: roughly 0.002 ETH/BNB. */
const EVM_GAS_BUFFER_WEI = 2_000_000_000_000_000n;

export function hasEvmWallet(): boolean {
  return typeof window !== "undefined" && Boolean(window.ethereum);
}

export function hasSolanaWallet(): boolean {
  return typeof window !== "undefined" && Boolean(window.solana);
}

export function walletAvailable(kind: WalletKind): boolean {
  if (kind === "evm") return hasEvmWallet();
  if (kind === "solana") return hasSolanaWallet();
  return false;
}

/* ------------------------------------------------------------------ */
/* Formatting                                                         */
/* ------------------------------------------------------------------ */

/** Render a base-unit amount as a decimal string. Pure integer arithmetic. */
export function formatUnits(value: bigint, decimals: number, maxFractionDigits = 6): string {
  const negative = value < 0n;
  const abs = negative ? -value : value;
  const scale = 10n ** BigInt(decimals);
  const whole = abs / scale;
  let fraction = (abs % scale).toString().padStart(decimals, "0");
  if (maxFractionDigits < decimals) fraction = fraction.slice(0, maxFractionDigits);
  fraction = fraction.replace(/0+$/, "");
  const body = fraction ? `${whole}.${fraction}` : whole.toString();
  return negative ? `-${body}` : body;
}

/** Parse a base-unit decimal string into a bigint, rejecting anything malformed. */
export function parseBaseUnits(value: string): bigint | null {
  if (!/^\d+$/.test(value.trim())) return null;
  try {
    return BigInt(value.trim());
  } catch {
    return null;
  }
}

function hexToBigInt(hex: unknown): bigint {
  if (typeof hex !== "string" || !hex.startsWith("0x")) return 0n;
  try {
    return BigInt(hex);
  } catch {
    return 0n;
  }
}

function errorMessage(err: unknown, fallback: string): string {
  if (err && typeof err === "object" && "message" in err) {
    const message = (err as { message?: unknown }).message;
    if (typeof message === "string" && message.trim()) return message;
  }
  return fallback;
}

/* ------------------------------------------------------------------ */
/* EVM                                                                */
/* ------------------------------------------------------------------ */

async function evmConnect(): Promise<ConnectedWallet> {
  const provider = window.ethereum;
  if (!provider) throw new Error("No EVM wallet detected. Install MetaMask or a compatible wallet.");

  const accounts = (await provider.request({ method: "eth_requestAccounts" })) as string[];
  const address = accounts?.[0];
  if (!address) throw new Error("The wallet returned no account.");

  const chainIdHex = (await provider.request({ method: "eth_chainId" })) as string;
  return { kind: "evm", address, chainId: Number(hexToBigInt(chainIdHex)) };
}

/**
 * Ask the wallet to switch networks.
 *
 * The caller must re-read the chain id afterwards rather than assuming success:
 * a user can dismiss the prompt, and some wallets resolve the request without
 * actually switching.
 */
export async function switchEvmChain(chainId: number): Promise<void> {
  const provider = window.ethereum;
  if (!provider) throw new Error("No EVM wallet detected.");
  await provider.request({
    method: "wallet_switchEthereumChain",
    params: [{ chainId: `0x${chainId.toString(16)}` }],
  });
}

export async function readEvmChainId(): Promise<number | null> {
  const provider = window.ethereum;
  if (!provider) return null;
  try {
    const hex = (await provider.request({ method: "eth_chainId" })) as string;
    return Number(hexToBigInt(hex));
  } catch {
    return null;
  }
}

async function evmNativeBalance(address: string): Promise<bigint> {
  const provider = window.ethereum!;
  const raw = await provider.request({ method: "eth_getBalance", params: [address, "latest"] });
  return hexToBigInt(raw);
}

/** `balanceOf(address)` — selector 0x70a08231, argument left-padded to 32 bytes. */
async function evmErc20Balance(contract: string, address: string): Promise<bigint> {
  const provider = window.ethereum!;
  const data = `0x70a08231${address.replace(/^0x/, "").toLowerCase().padStart(64, "0")}`;
  const raw = await provider.request({
    method: "eth_call",
    params: [{ to: contract, data }, "latest"],
  });
  return hexToBigInt(raw);
}

/* ------------------------------------------------------------------ */
/* Solana                                                             */
/* ------------------------------------------------------------------ */

async function solanaConnect(): Promise<ConnectedWallet> {
  const provider = window.solana;
  if (!provider) throw new Error("No Solana wallet detected. Install Phantom or a compatible wallet.");
  const { publicKey } = await provider.connect();
  return { kind: "solana", address: publicKey.toString(), chainId: null };
}

async function solanaRpc<T>(endpoint: string, method: string, params: unknown[]): Promise<T> {
  const response = await fetch(endpoint, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
  });
  if (!response.ok) throw new Error(`Solana RPC returned HTTP ${response.status}`);
  const json = (await response.json()) as { result?: T; error?: { message?: string } };
  if (json.error) throw new Error(json.error.message ?? "Solana RPC error");
  if (json.result === undefined) throw new Error("Solana RPC returned no result");
  return json.result;
}

async function solanaLamports(endpoint: string, address: string): Promise<bigint> {
  const result = await solanaRpc<{ value: number }>(endpoint, "getBalance", [
    address,
    { commitment: "confirmed" },
  ]);
  return BigInt(result.value ?? 0);
}

/* ------------------------------------------------------------------ */
/* Public API                                                         */
/* ------------------------------------------------------------------ */

export async function connectWallet(kind: WalletKind): Promise<ConnectedWallet> {
  if (kind === "evm") return evmConnect();
  if (kind === "solana") return solanaConnect();
  throw new Error("This payment route has no browser wallet integration.");
}

export async function disconnectWallet(kind: WalletKind): Promise<void> {
  // EIP-1193 has no disconnect; the user revokes access in the wallet itself.
  if (kind === "solana") {
    try {
      await window.solana?.disconnect?.();
    } catch {
      /* a wallet that refuses to disconnect is not an error worth surfacing */
    }
  }
}

/**
 * Read the connected wallet's balance and compare it with `requiredBaseUnits`.
 *
 * Call this immediately before letting the user sign, not only when the page
 * loads: balances move, and a stale "you have enough" is worse than no check.
 */
export async function checkBalance(
  route: PaymentRoute,
  wallet: ConnectedWallet,
  requiredBaseUnits: bigint,
): Promise<BalanceCheck> {
  if (route.walletKind === "manual") {
    return {
      state: "unsupported",
      reason: `${route.token} has no browser wallet standard, so the balance cannot be checked here. Send the exact amount from your own wallet.`,
    };
  }

  try {
    if (route.walletKind === "evm") {
      if (route.chainId !== null && wallet.chainId !== route.chainId) {
        return {
          state: "error",
          reason: `Wallet is on chain ${wallet.chainId ?? "unknown"} but this payment settles on chain ${route.chainId}. Switch networks first.`,
        };
      }

      const native = await evmNativeBalance(wallet.address);
      const balance = route.tokenContract
        ? await evmErc20Balance(route.tokenContract, wallet.address)
        : native;

      // On a native route the gas buffer comes out of the same balance, so the
      // user needs amount + gas; on an ERC-20 route they need gas separately.
      const needed = route.tokenContract ? requiredBaseUnits : requiredBaseUnits + EVM_GAS_BUFFER_WEI;
      const sufficient = balance >= needed;

      return {
        state: "ok",
        balance,
        required: requiredBaseUnits,
        sufficient,
        shortfall: sufficient ? 0n : needed - balance,
        nativeBalance: native,
        gasLooksThin: native < EVM_GAS_BUFFER_WEI,
      };
    }

    // Solana: native SOL only (the backend does not settle SPL tokens on Solana).
    if (!route.rpcUrl) {
      return {
        state: "error",
        reason:
          "The server did not supply a Solana RPC endpoint for this route, so the balance cannot be read on the same cluster the payment will be verified on.",
      };
    }
    const lamports = await solanaLamports(route.rpcUrl, wallet.address);
    const needed = requiredBaseUnits + SOLANA_FEE_BUFFER_LAMPORTS;
    const sufficient = lamports >= needed;

    return {
      state: "ok",
      balance: lamports,
      required: requiredBaseUnits,
      sufficient,
      shortfall: sufficient ? 0n : needed - lamports,
      nativeBalance: lamports,
      gasLooksThin: lamports < SOLANA_FEE_BUFFER_LAMPORTS,
    };
  } catch (err) {
    return { state: "error", reason: errorMessage(err, "Could not read the wallet balance.") };
  }
}

/** Shorten an address for display: `0x742d…bEb0`. */
export function shortenAddress(address: string): string {
  if (address.length <= 12) return address;
  return `${address.slice(0, 6)}…${address.slice(-4)}`;
}

/**
 * Subscribe to account and network changes.
 *
 * A user switching accounts or networks mid-checkout must invalidate the balance
 * check; silently keeping the old numbers is how someone ends up paying from the
 * wrong wallet or on the wrong chain.
 */
export function watchWallet(kind: WalletKind, onChange: () => void): () => void {
  if (kind === "evm" && window.ethereum?.on) {
    const provider = window.ethereum;
    const handler = () => onChange();
    provider.on?.("accountsChanged", handler);
    provider.on?.("chainChanged", handler);
    return () => {
      provider.removeListener?.("accountsChanged", handler);
      provider.removeListener?.("chainChanged", handler);
    };
  }
  if (kind === "solana" && window.solana?.on) {
    const provider = window.solana;
    const handler = () => onChange();
    provider.on?.("accountChanged", handler);
    provider.on?.("disconnect", handler);
    return () => {
      provider.removeListener?.("accountChanged", handler);
      provider.removeListener?.("disconnect", handler);
    };
  }
  return () => {};
}
