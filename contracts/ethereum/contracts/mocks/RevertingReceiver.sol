// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @notice A contract that rejects plain ETH transfers (no receive/fallback), used to model
 *         a recipient whose payout would fail. Demonstrates that the escrow's checked
 *         low-level call surfaces the failure via a revert ("ETH transfer failed") rather
 *         than silently losing funds. In the test it is impersonated to originate an order,
 *         then refuses the refund payout.
 */
contract RevertingReceiver {
    // Intentionally no receive() / fallback() so any inbound ETH send fails.
}
