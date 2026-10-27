import type { AssetCatalogItem } from "./backend";

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
  backendAssetId?: string;
};

const CART_STORAGE_KEY = "aeroledger-cart";

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

export function addBackendAssetToCart(asset: AssetCatalogItem): CartItem[] {
  const usd = `$${asset.token_price.toLocaleString(undefined, { maximumFractionDigits: 0 })}`;
  const nextItem: CartItem = {
    id: `asset-${asset.id}`,
    sourceId: asset.id,
    kind: "asset",
    category: asset.type,
    name: asset.name,
    subtitle: asset.description ?? "Tokenized luxury asset",
    primaryMeta: `${asset.available_tokens.toLocaleString()} of ${asset.total_tokens.toLocaleString()} tokens available`,
    secondaryMeta: `Token price ${usd}`,
    cryptoPrice: usd,
    fiatPrice: usd,
    image: asset.image ?? undefined,
    backendAssetId: asset.id,
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
