import { beforeEach, describe, expect, it } from "vitest";

import {
  addBackendAssetToCart,
  addFlightToCart,
  cartTotalSummary,
  getCartItems,
  removeCartItem,
} from "./commerce";
import type { AssetCatalogItem } from "./backend";

const STORAGE_KEY = "aeroledger-cart";

function asset(overrides: Partial<AssetCatalogItem> = {}): AssetCatalogItem {
  return {
    id: "00000000-0000-0000-0000-00000000a003",
    name: "Private Terminal Lounge Access",
    type: "membership",
    description: "Tokenized annual private-terminal lounge membership.",
    image: null,
    total_value: 4_200_000,
    token_price: 4_200,
    available_tokens: 1_000,
    total_tokens: 1_000,
    status: "active",
    ...overrides,
  };
}

beforeEach(() => {
  window.localStorage.clear();
});

describe("addBackendAssetToCart", () => {
  /**
   * The regression that matters most. The asset detail page used to build cart
   * items from a hardcoded catalogue and omit the backend id entirely, so
   * checkout sent `asset_id: undefined` and the booking could never be created.
   * Any cart item for an asset must carry the id the server knows it by.
   */
  it("carries the backend asset id through to the cart", () => {
    const items = addBackendAssetToCart(asset());

    expect(items).toHaveLength(1);
    expect(items[0].backendAssetId).toBe("00000000-0000-0000-0000-00000000a003");
    expect(items[0].kind).toBe("asset");
  });

  it("keys the cart item on the backend id so it is stable across sessions", () => {
    const items = addBackendAssetToCart(asset());
    expect(items[0].id).toBe("asset-00000000-0000-0000-0000-00000000a003");
    expect(items[0].sourceId).toBe("00000000-0000-0000-0000-00000000a003");
  });

  it("does not add the same asset twice", () => {
    addBackendAssetToCart(asset());
    const items = addBackendAssetToCart(asset());
    expect(items).toHaveLength(1);
  });

  it("adds distinct assets separately", () => {
    addBackendAssetToCart(asset());
    const items = addBackendAssetToCart(
      asset({ id: "00000000-0000-0000-0000-00000000a001", name: "Gulfstream G700" })
    );
    expect(items).toHaveLength(2);
    expect(new Set(items.map((i) => i.backendAssetId)).size).toBe(2);
  });

  it("persists to localStorage so a reload keeps the cart", () => {
    addBackendAssetToCart(asset());
    expect(getCartItems()[0].backendAssetId).toBe(
      "00000000-0000-0000-0000-00000000a003"
    );
  });

  it("formats the displayed price from the server's token price", () => {
    const items = addBackendAssetToCart(asset({ token_price: 15_500 }));
    expect(items[0].fiatPrice).toBe("$15,500");
  });
});

describe("addFlightToCart", () => {
  const flight = {
    category: "Charter",
    name: "London to Dubai",
    subtitle: "Airbus A380",
    primaryMeta: "LHR-DXB",
    secondaryMeta: "7h 15m",
    cryptoPrice: "$9,775",
    fiatPrice: "$9,775",
    backendFlightId: "00000000-0000-0000-0000-000000000003",
  };

  it("keeps the backend flight id on the cart item", () => {
    const items = addFlightToCart(flight);
    expect(items[0].backendFlightId).toBe("00000000-0000-0000-0000-000000000003");
    expect(items[0].kind).toBe("flight");
  });

  it("does not add the same flight twice", () => {
    addFlightToCart(flight);
    expect(addFlightToCart(flight)).toHaveLength(1);
  });

  it("keeps a flight and an asset as separate items", () => {
    addFlightToCart(flight);
    const items = addBackendAssetToCart(asset());
    expect(items).toHaveLength(2);
    expect(items.map((i) => i.kind).sort()).toEqual(["asset", "flight"]);
  });
});

describe("removeCartItem", () => {
  it("removes only the named item", () => {
    addBackendAssetToCart(asset());
    addBackendAssetToCart(
      asset({ id: "00000000-0000-0000-0000-00000000a001", name: "Gulfstream G700" })
    );

    const items = removeCartItem("asset-00000000-0000-0000-0000-00000000a003");
    expect(items).toHaveLength(1);
    expect(items[0].backendAssetId).toBe("00000000-0000-0000-0000-00000000a001");
  });

  it("is a no-op for an id that is not in the cart", () => {
    addBackendAssetToCart(asset());
    expect(removeCartItem("asset-does-not-exist")).toHaveLength(1);
  });
});

describe("cart storage robustness", () => {
  /**
   * localStorage is shared with whatever else runs on this origin and survives
   * deploys. Malformed content must not take the cart page down with it.
   */
  it("recovers from malformed stored JSON instead of throwing", () => {
    window.localStorage.setItem(STORAGE_KEY, "{not json");
    expect(getCartItems()).toEqual([]);
  });

  it("recovers from stored JSON that is not an array", () => {
    window.localStorage.setItem(STORAGE_KEY, '{"cart":"nope"}');
    expect(getCartItems()).toEqual([]);
  });

  it("returns an empty cart when nothing is stored", () => {
    expect(getCartItems()).toEqual([]);
  });
});

describe("cartTotalSummary", () => {
  it("counts items and previews their prices", () => {
    addBackendAssetToCart(asset());
    const summary = cartTotalSummary(getCartItems());
    expect(summary.itemCount).toBe(1);
    expect(summary.preview).toBe("$4,200");
  });

  it("is empty for an empty cart", () => {
    expect(cartTotalSummary([])).toEqual({ itemCount: 0, preview: "" });
  });
});
