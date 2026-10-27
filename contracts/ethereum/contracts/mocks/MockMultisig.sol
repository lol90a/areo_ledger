// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @notice Minimal multisig/Gnosis-Safe-style contract owner. Its receive() runs more than
 *         the 2300-gas stipend that `.transfer()` forwards (it writes storage), so a payout
 *         using `.transfer()` would revert. This proves the escrow's low-level `.call`-based
 *         payout keeps working when the platform owner is a smart-contract wallet.
 *
 * @dev Can own the escrow and forward owner-only calls (confirm/complete/etc.) to it.
 */
contract MockMultisig {
    uint256 public received;   // storage write in receive() => exceeds 2300 gas stipend
    uint256 public pings;

    receive() external payable {
        received += msg.value;
        pings += 1;
    }

    /// @notice Execute an arbitrary call (used to drive owner-only escrow functions).
    function exec(address target, bytes calldata data) external returns (bytes memory) {
        (bool ok, bytes memory ret) = target.call(data);
        require(ok, "exec failed");
        return ret;
    }
}
