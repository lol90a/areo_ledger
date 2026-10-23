// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @notice USDT-style token that intentionally does NOT return a bool from transfer /
 *         transferFrom / approve. This is the exact non-standard behavior of the real
 *         USDT (0xdAC17F958D2ee523a2206206994597C13D831ec7) that breaks naive
 *         `IERC20(token).transfer(...)` calls under Solidity 0.8.20 ABI return-decoding.
 *         Used to prove the escrow's SafeERC20 usage handles it correctly.
 *
 * @dev Deliberately a hand-rolled implementation (not OpenZeppelin ERC20) so the
 *      function signatures omit the `returns (bool)`.
 */
contract MockUSDT {
    string public name = "Tether USD";
    string public symbol = "USDT";
    uint8 public decimals = 6;
    uint256 public totalSupply;

    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

    constructor(uint256 initialSupply) {
        totalSupply = initialSupply;
        balanceOf[msg.sender] = initialSupply;
        emit Transfer(address(0), msg.sender, initialSupply);
    }

    function mint(address to, uint256 amount) external {
        totalSupply += amount;
        balanceOf[to] += amount;
        emit Transfer(address(0), to, amount);
    }

    // NOTE: no `returns (bool)` on purpose.
    function transfer(address to, uint256 value) external {
        require(balanceOf[msg.sender] >= value, "USDT: insufficient balance");
        balanceOf[msg.sender] -= value;
        balanceOf[to] += value;
        emit Transfer(msg.sender, to, value);
    }

    // NOTE: no `returns (bool)` on purpose.
    function approve(address spender, uint256 value) external {
        allowance[msg.sender][spender] = value;
        emit Approval(msg.sender, spender, value);
    }

    // NOTE: no `returns (bool)` on purpose.
    function transferFrom(address from, address to, uint256 value) external {
        uint256 allowed = allowance[from][msg.sender];
        require(allowed >= value, "USDT: insufficient allowance");
        require(balanceOf[from] >= value, "USDT: insufficient balance");
        if (allowed != type(uint256).max) {
            allowance[from][msg.sender] = allowed - value;
        }
        balanceOf[from] -= value;
        balanceOf[to] += value;
        emit Transfer(from, to, value);
    }
}
