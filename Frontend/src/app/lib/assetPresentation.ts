import type { AssetCatalogItem } from "./backend";

/**
 * Shown when the catalogue row has no image of its own. Every seeded asset
 * currently has a null image, so this is the normal path, not an edge case.
 */
export const ASSET_FALLBACK_IMAGE =
  "https://images.unsplash.com/photo-1540962351504-03099e0a754b?auto=format&fit=crop&w=1080&q=80";

export function formatUsd(value: number) {
  return `$${value.toLocaleString(undefined, { maximumFractionDigits: 0 })}`;
}

export function assetImage(asset: Pick<AssetCatalogItem, "image">) {
  return asset.image ?? ASSET_FALLBACK_IMAGE;
}
