import { describe, expect, it } from "vitest";

import { normalizeSettlementStatus } from "./backend";
import { assetImage, formatUsd, ASSET_FALLBACK_IMAGE } from "./assetPresentation";

/**
 * The checkout's status strip is driven entirely by this mapping. Getting it
 * wrong in the optimistic direction is the serious case: showing "confirmed"
 * for a payment the chain has not settled is exactly the defect the original
 * frontend shipped, where success came from a three-second timer.
 */
describe("normalizeSettlementStatus", () => {
  it("maps confirmed through unchanged", () => {
    expect(normalizeSettlementStatus("confirmed")).toBe("confirmed");
  });

  it("treats cancelled and failed as failure, both spellings", () => {
    expect(normalizeSettlementStatus("failed")).toBe("failed");
    expect(normalizeSettlementStatus("cancelled")).toBe("failed");
    expect(normalizeSettlementStatus("canceled")).toBe("failed");
  });

  it("maps pending to awaiting, not to confirmed", () => {
    expect(normalizeSettlementStatus("pending")).toBe("awaiting");
  });

  it("keeps initialized as its own step", () => {
    expect(normalizeSettlementStatus("initialized")).toBe("initialized");
  });

  it("is case-insensitive, since the backend's casing is not a contract", () => {
    expect(normalizeSettlementStatus("CONFIRMED")).toBe("confirmed");
    expect(normalizeSettlementStatus("Pending")).toBe("awaiting");
  });

  /**
   * The decisive property: anything unrecognised must fall back to "awaiting",
   * never to "confirmed". An unknown status means we do not know the payment
   * settled, and the UI must not claim that it did.
   */
  it("falls back to awaiting for unknown, null, undefined and empty input", () => {
    for (const input of ["", "   ", "weird-new-status", null, undefined]) {
      expect(normalizeSettlementStatus(input)).toBe("awaiting");
    }
  });

  it("never invents a confirmed state from unrecognised input", () => {
    const inputs = ["success", "ok", "done", "complete", "settled", "paid"];
    for (const input of inputs) {
      expect(
        normalizeSettlementStatus(input),
        `"${input}" must not read as confirmed`
      ).not.toBe("confirmed");
    }
  });
});

describe("formatUsd", () => {
  it("groups thousands and drops cents", () => {
    expect(formatUsd(4_200)).toBe("$4,200");
    expect(formatUsd(78_000_000)).toBe("$78,000,000");
  });

  it("handles zero", () => {
    expect(formatUsd(0)).toBe("$0");
  });
});

describe("assetImage", () => {
  it("uses the catalogue image when there is one", () => {
    expect(assetImage({ image: "https://example.com/jet.jpg" })).toBe(
      "https://example.com/jet.jpg"
    );
  });

  /** Every seeded asset currently has a null image, so this is the normal path. */
  it("falls back when the catalogue image is null", () => {
    expect(assetImage({ image: null })).toBe(ASSET_FALLBACK_IMAGE);
  });
});
