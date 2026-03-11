// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract AeroLedgerEscrow is ReentrancyGuard, Ownable {
    enum OrderStatus { Pending, Confirmed, Completed, Refunded, Cancelled }
    
    struct Order {
        address buyer;
        uint256 amount;
        address token; // address(0) for ETH/BNB
        OrderStatus status;
        uint256 createdAt;
        string orderId; // Backend order ID
    }
    
    mapping(bytes32 => Order) public orders;
    mapping(address => bool) public supportedTokens;
    
    uint256 public constant REFUND_WINDOW = 24 hours;
    
    event OrderCreated(bytes32 indexed orderHash, address buyer, uint256 amount, address token, string orderId);
    event OrderConfirmed(bytes32 indexed orderHash);
    event OrderCompleted(bytes32 indexed orderHash);
    event OrderRefunded(bytes32 indexed orderHash, uint256 amount);
    event OrderCancelled(bytes32 indexed orderHash);
    
    constructor() Ownable(msg.sender) {
        // ETH/BNB native token
        supportedTokens[address(0)] = true;
    }
    
    function addSupportedToken(address token) external onlyOwner {
        supportedTokens[token] = true;
    }
    
    function createOrder(string calldata orderId, address token) external payable nonReentrant {
        require(supportedTokens[token], "Token not supported");
        require(msg.value > 0, "Amount must be > 0");
        
        bytes32 orderHash = keccak256(abi.encodePacked(msg.sender, orderId, block.timestamp));
        require(orders[orderHash].buyer == address(0), "Order exists");
        
        orders[orderHash] = Order({
            buyer: msg.sender,
            amount: msg.value,
            token: token,
            status: OrderStatus.Pending,
            createdAt: block.timestamp,
            orderId: orderId
        });
        
        emit OrderCreated(orderHash, msg.sender, msg.value, token, orderId);
    }
    
    function createOrderERC20(string calldata orderId, address token, uint256 amount) external nonReentrant {
        require(supportedTokens[token], "Token not supported");
        require(token != address(0), "Use createOrder for native");
        require(amount > 0, "Amount must be > 0");
        
        bytes32 orderHash = keccak256(abi.encodePacked(msg.sender, orderId, block.timestamp));
        require(orders[orderHash].buyer == address(0), "Order exists");
        
        IERC20(token).transferFrom(msg.sender, address(this), amount);
        
        orders[orderHash] = Order({
            buyer: msg.sender,
            amount: amount,
            token: token,
            status: OrderStatus.Pending,
            createdAt: block.timestamp,
            orderId: orderId
        });
        
        emit OrderCreated(orderHash, msg.sender, amount, token, orderId);
    }
    
    function confirmOrder(bytes32 orderHash) external onlyOwner {
        Order storage order = orders[orderHash];
        require(order.status == OrderStatus.Pending, "Invalid status");
        
        order.status = OrderStatus.Confirmed;
        emit OrderConfirmed(orderHash);
    }
    
    function completeOrder(bytes32 orderHash) external onlyOwner nonReentrant {
        Order storage order = orders[orderHash];
        require(order.status == OrderStatus.Confirmed, "Not confirmed");
        
        order.status = OrderStatus.Completed;
        
        if (order.token == address(0)) {
            payable(owner()).transfer(order.amount);
        } else {
            IERC20(order.token).transfer(owner(), order.amount);
        }
        
        emit OrderCompleted(orderHash);
    }
    
    function refundOrder(bytes32 orderHash) external nonReentrant {
        Order storage order = orders[orderHash];
        require(order.buyer == msg.sender || msg.sender == owner(), "Not authorized");
        require(order.status == OrderStatus.Pending || order.status == OrderStatus.Confirmed, "Cannot refund");
        require(block.timestamp <= order.createdAt + REFUND_WINDOW, "Refund window expired");
        
        order.status = OrderStatus.Refunded;
        
        if (order.token == address(0)) {
            payable(order.buyer).transfer(order.amount);
        } else {
            IERC20(order.token).transfer(order.buyer, order.amount);
        }
        
        emit OrderRefunded(orderHash, order.amount);
    }
    
    function cancelOrder(bytes32 orderHash) external onlyOwner {
        Order storage order = orders[orderHash];
        require(order.status == OrderStatus.Pending, "Cannot cancel");
        
        order.status = OrderStatus.Cancelled;
        
        if (order.token == address(0)) {
            payable(order.buyer).transfer(order.amount);
        } else {
            IERC20(order.token).transfer(order.buyer, order.amount);
        }
        
        emit OrderCancelled(orderHash);
    }
    
    function getOrder(bytes32 orderHash) external view returns (Order memory) {
        return orders[orderHash];
    }
}
