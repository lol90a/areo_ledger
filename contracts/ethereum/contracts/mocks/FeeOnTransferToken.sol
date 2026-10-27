// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

/// @notice ERC20 that burns a fee on every transfer, so the recipient receives less than
///         the sent amount. Used to prove the escrow credits the actual received balance
///         delta rather than the requested amount.
contract FeeOnTransferToken is ERC20 {
    uint256 public immutable feeBps; // fee in basis points (e.g. 100 = 1%)

    constructor(uint256 feeBps_, uint256 initialSupply) ERC20("FeeToken", "FEE") {
        feeBps = feeBps_;
        _mint(msg.sender, initialSupply);
    }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }

    function _update(address from, address to, uint256 value) internal override {
        if (from != address(0) && to != address(0) && feeBps > 0) {
            uint256 fee = (value * feeBps) / 10_000;
            if (fee > 0) {
                super._update(from, address(0), fee); // burn the fee
                super._update(from, to, value - fee);
                return;
            }
        }
        super._update(from, to, value);
    }
}
