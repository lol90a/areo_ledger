import { describe, expect, it } from "vitest";

import { formatUnits, parseBaseUnits } from "./wallet";

/**
 * The money math. Every amount the checkout displays or sends is produced by
 * these two functions, and both work in the token's smallest unit as a bigint —
 * no float ever touches a balance or a payment amount. These tests exist to keep
 * it that way: a rounding bug here is a wrong payment, not a cosmetic defect.
 */
describe("formatUnits", () => {
  it("renders whole and fractional parts at the token's own precision", () => {
    // 97.818472932 SOL, the exact figure a real settlement produced.
    expect(formatUnits(97_818_472_932n, 9, 9)).toBe("97.818472932");
  });

  it("renders a whole number without a trailing point", () => {
    expect(formatUnits(5_000_000_000n, 9)).toBe("5");
  });

  it("renders a value smaller than one unit", () => {
    expect(formatUnits(1n, 9, 9)).toBe("0.000000001");
  });

  it("pads the fraction so leading zeros are not lost", () => {
    // Without left-padding this would read 0.1 instead of 0.000000001.
    expect(formatUnits(1n, 9, 9)).not.toBe("0.1");
    expect(formatUnits(1_000n, 9, 9)).toBe("0.000001");
  });

  it("truncates rather than rounds when precision is capped", () => {
    // Rounding up here would display more than the user actually holds.
    expect(formatUnits(97_818_472_932n, 9, 2)).toBe("97.81");
    expect(formatUnits(1_999_999_999n, 9, 2)).toBe("1.99");
  });

  it("handles the 18-decimal case without precision loss", () => {
    // A Number would have lost the tail of this long before here.
    expect(formatUnits(1_234_567_890_123_456_789n, 18, 18)).toBe(
      "1.234567890123456789"
    );
  });

  it("handles zero", () => {
    expect(formatUnits(0n, 9)).toBe("0");
  });

  it("keeps the sign on a negative value", () => {
    expect(formatUnits(-1_500_000_000n, 9, 9)).toBe("-1.5");
  });
});

describe("parseBaseUnits", () => {
  it("accepts a plain integer string", () => {
    expect(parseBaseUnits("97818472932")).toBe(97_818_472_932n);
  });

  it("accepts a value far beyond Number.MAX_SAFE_INTEGER", () => {
    const huge = "123456789012345678901234567890";
    expect(parseBaseUnits(huge)).toBe(BigInt(huge));
  });

  it("tolerates surrounding whitespace", () => {
    expect(parseBaseUnits("  42  ")).toBe(42n);
  });

  /**
   * Base units are integers by definition. A decimal point here means the caller
   * has already mixed up display units with base units, and silently truncating
   * would turn 1.5 SOL into 1 lamport.
   */
  it("rejects a decimal string rather than truncating it", () => {
    expect(parseBaseUnits("1.5")).toBeNull();
    expect(parseBaseUnits("97.818472932")).toBeNull();
  });

  it("rejects anything that is not a base-ten integer", () => {
    for (const bad of ["", "   ", "abc", "0x1f", "-1", "1e9", "1,000", "+5"]) {
      expect(parseBaseUnits(bad), `"${bad}" must be rejected`).toBeNull();
    }
  });
});

describe("formatUnits and parseBaseUnits round-trip", () => {
  it("survives a round trip at full precision", () => {
    const original = 48_333_833_684n;
    const shown = formatUnits(original, 9, 9);
    // Display form is not a base-unit string, so it must not parse back.
    expect(parseBaseUnits(shown)).toBeNull();
    // The base-unit string does.
    expect(parseBaseUnits(original.toString())).toBe(original);
  });
});
