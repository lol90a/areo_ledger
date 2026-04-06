import { ASSET_DATA } from "../data/assets";

export type CartItem = {
  id: string;
  sourceId?: string;
  kind: "flight" | "asset";
  category: string;
  name: string;
  subtitle: string;
  primaryMeta: string;
  secondaryMeta: string;
  cryptoPrice: string;
  fiatPrice: string;
  image?: string;
  backendFlightId?: string;
};

export type OrderStatus = "initialized" | "awaiting" | "confirmed" | "failed";

export type OrderTimelineItem = {
  key: string;
  label: string;
  detail: string;
  status: "complete" | "current" | "upcoming" | "failed";
};

export type OrderRecord = {
  id: string;
  type: string;
  title: string;
  asset: string;
  network: string;
  amount: string;
  fiatAmount: string;
  status: OrderStatus;
  date: string;
  txHash: string;
  walletAddress: string;
  documents: { name: string; type: string }[];
  manager: {
    name: string;
    role: string;
    email: string;
    telegram: string;
  };
  timeline: OrderTimelineItem[];
};

const CART_STORAGE_KEY = "aeroledger-cart";

const ORDER_RECORDS: OrderRecord[] = [
  {
    id: "AL-9401-FX",
    type: "Flight Booking",
    title: "Challenger 350 Charter",
    asset: "USDC",
    network: "Ethereum",
    amount: "45,000 USDC",
    fiatAmount: "$45,000",
    status: "confirmed",
    date: "Apr 4, 2026",
    txHash: "0x8fB1...4e21",
    walletAddress: "0xAeroLedgerTreasury001",
    documents: [
      { name: "Itinerary.pdf", type: "Travel dossier" },
      { name: "PassengerManifest.pdf", type: "Identity package" },
      { name: "CryptoSettlementMemo.pdf", type: "Settlement memo" },
    ],
    manager: {
      name: "Lina Mercer",
      role: "Transaction Manager",
      email: "lina@aeroledger.com",
      telegram: "@lina_aeroledger",
    },
    timeline: [
      { key: "initialized", label: "Payment initialized", detail: "Wallet instructions generated for Ethereum USDC settlement.", status: "complete" },
      { key: "awaiting", label: "Awaiting confirmations", detail: "Transaction hash received and blockchain monitoring completed.", status: "complete" },
      { key: "confirmed", label: "Payment confirmed", detail: "Funds reconciled and booking released to flight operations.", status: "complete" },
    ],
  },
  {
    id: "AL-9402-AC",
    type: "Aircraft Acquisition",
    title: "Gulfstream G650ER",
    asset: "BTC",
    network: "Bitcoin",
    amount: "845.2 BTC",
    fiatAmount: "$56,500,000",
    status: "awaiting",
    date: "Apr 3, 2026",
    txHash: "bc1q...x0v9",
    walletAddress: "bc1qaeroledgerescrow001",
    documents: [
      { name: "LOI.pdf", type: "Letter of intent" },
      { name: "EscrowSchedule.pdf", type: "Escrow schedule" },
      { name: "TitleTransferChecklist.pdf", type: "Transfer checklist" },
    ],
    manager: {
      name: "Marcus Vale",
      role: "Escrow Director",
      email: "marcus@aeroledger.com",
      telegram: "@marcus_escrow",
    },
    timeline: [
      { key: "initialized", label: "Payment initialized", detail: "Escrow wallet and settlement instructions shared with buyer counsel.", status: "complete" },
      { key: "awaiting", label: "Awaiting confirmations", detail: "Transaction hash submitted. Monitoring confirmations and compliance checks.", status: "current" },
      { key: "confirmed", label: "Payment confirmed", detail: "Aircraft title release will trigger after final treasury sign-off.", status: "upcoming" },
    ],
  },
  {
    id: "AL-9403-FX",
    type: "Flight Booking",
    title: "Citation Latitude Charter",
    asset: "SOL",
    network: "Solana",
    amount: "150 SOL",
    fiatAmount: "$22,000",
    status: "initialized",
    date: "Apr 2, 2026",
    txHash: "Pending submission",
    walletAddress: "SoL4AeroLedgerTreasury001",
    documents: [
      { name: "ReservationDraft.pdf", type: "Booking draft" },
      { name: "RouteApproval.pdf", type: "Dispatch approval" },
    ],
    manager: {
      name: "Nadia Ford",
      role: "Client Concierge",
      email: "nadia@aeroledger.com",
      telegram: "@nadia_concierge",
    },
    timeline: [
      { key: "initialized", label: "Payment initialized", detail: "Reservation is ready. Send SOL to secure the aircraft slot.", status: "current" },
      { key: "awaiting", label: "Awaiting confirmations", detail: "Submit your transaction hash to trigger chain monitoring.", status: "upcoming" },
      { key: "confirmed", label: "Payment confirmed", detail: "Flight operations receives final release after settlement confirmation.", status: "upcoming" },
    ],
  },
  {
    id: "AL-9404-AC",
    type: "Property Purchase",
    title: "Skyline Penthouse",
    asset: "ETH",
    network: "Ethereum",
    amount: "420.5 ETH",
    fiatAmount: "$28,500,000",
    status: "failed",
    date: "Mar 30, 2026",
    txHash: "0x1a...f99b",
    walletAddress: "0xAeroLedgerEscrowFailed001",
    documents: [
      { name: "KYCReview.pdf", type: "Compliance review" },
      { name: "FundingException.pdf", type: "Exception note" },
    ],
    manager: {
      name: "Sofia Lin",
      role: "Settlement Operations",
      email: "sofia@aeroledger.com",
      telegram: "@sofia_ops",
    },
    timeline: [
      { key: "initialized", label: "Payment initialized", detail: "Funding instructions were created for escrow settlement.", status: "complete" },
      { key: "awaiting", label: "Awaiting confirmations", detail: "Transaction submitted but confirmation rules were not satisfied.", status: "failed" },
      { key: "confirmed", label: "Payment confirmed", detail: "Order remains on hold pending treasury review.", status: "upcoming" },
    ],
  },
];

function isBrowser() {
  return typeof window !== "undefined";
}

function dispatchCartUpdate() {
  if (isBrowser()) {
    window.dispatchEvent(new Event("aeroledger-cart-updated"));
  }
}

function readCartFromStorage(): CartItem[] {
  if (!isBrowser()) {
    return [];
  }

  const raw = window.localStorage.getItem(CART_STORAGE_KEY);
  if (!raw) {
    return [];
  }

  try {
    const parsed = JSON.parse(raw) as CartItem[];
    if (Array.isArray(parsed)) {
      return parsed;
    }
  } catch (_error) {
    // Ignore malformed storage and restore an empty cart state.
  }

  window.localStorage.removeItem(CART_STORAGE_KEY);
  return [];
}

function writeCartToStorage(items: CartItem[]) {
  if (!isBrowser()) {
    return;
  }

  window.localStorage.setItem(CART_STORAGE_KEY, JSON.stringify(items));
  dispatchCartUpdate();
}

export function getCartItems(): CartItem[] {
  return readCartFromStorage();
}

export function addAssetToCart(assetId: string): CartItem[] {
  const asset = ASSET_DATA.find((entry) => entry.id === assetId);
  if (!asset) {
    return readCartFromStorage();
  }

  const nextItem: CartItem = {
    id: `asset-${asset.id}`,
    sourceId: asset.id,
    kind: "asset",
    category: asset.assetType,
    name: asset.name,
    subtitle: `${asset.manufacturer} - ${asset.category}`,
    primaryMeta: asset.location,
    secondaryMeta: `${asset.year} model`,
    cryptoPrice: asset.price,
    fiatPrice: asset.usdPrice,
    image: asset.image,
  };

  const current = readCartFromStorage();
  if (current.some((item) => item.id === nextItem.id)) {
    return current;
  }

  const updated = [...current, nextItem];
  writeCartToStorage(updated);
  return updated;
}

export function addFlightToCart(flight: Omit<CartItem, "id" | "kind">): CartItem[] {
  const current = readCartFromStorage();
  const nextItem: CartItem = {
    ...flight,
    id: `flight-${flight.name.toLowerCase().replace(/[^a-z0-9]+/g, "-")}`,
    kind: "flight",
  };

  if (current.some((item) => item.id === nextItem.id)) {
    return current;
  }

  const updated = [...current, nextItem];
  writeCartToStorage(updated);
  return updated;
}

export function removeCartItem(itemId: string): CartItem[] {
  const current = readCartFromStorage();
  const updated = current.filter((item) => item.id !== itemId);
  writeCartToStorage(updated);
  return updated;
}

export function cartTotalSummary(items: CartItem[]) {
  return {
    itemCount: items.length,
    preview: items.map((item) => item.cryptoPrice).join(" + "),
  };
}

export function getOrders(): OrderRecord[] {
  return ORDER_RECORDS;
}

export function getOrderById(orderId: string) {
  return ORDER_RECORDS.find((order) => order.id === orderId);
}
