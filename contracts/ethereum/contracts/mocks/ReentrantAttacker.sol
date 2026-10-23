// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IEscrow {
    struct QuoteInput {
        string orderId;
        address seller;
        address token;
        uint256 amount;
        uint256 fee;
        uint256 expiry;
        bytes signature;
    }

    function createOrder(QuoteInput calldata q) external payable;
    function refundOrder(bytes32 orderHash) external;
    function computeOrderHash(address buyer, string calldata orderId) external pure returns (bytes32);
}

/**
 * @notice Malicious buyer contract that attempts to re-enter {refundOrder} from its
 *         native receive() hook during a refund payout. The escrow's nonReentrant guard
 *         (plus CEI ordering + the escrow-balance ledger) must make the re-entrant call
 *         revert, so the attacker can never be paid twice.
 *
 * @dev The quote is signed off-chain for THIS contract's address as the buyer, which is
 *      how the test gets a legitimately-priced order into a contract that can re-enter.
 */
contract ReentrantAttacker {
    IEscrow public immutable escrow;
    bytes32 public targetHash;
    bool public reentered;
    bool public attackEnabled;

    constructor(address escrow_) {
        escrow = IEscrow(escrow_);
    }

    function createOrder(IEscrow.QuoteInput calldata q) external payable {
        targetHash = escrow.computeOrderHash(address(this), q.orderId);
        escrow.createOrder{value: msg.value}(q);
    }

    function attackRefund() external {
        attackEnabled = true;
        escrow.refundOrder(targetHash);
    }

    receive() external payable {
        if (attackEnabled && !reentered) {
            reentered = true;
            // Attempt the re-entrant refund; expected to revert inside the escrow.
            escrow.refundOrder(targetHash);
        }
    }
}
