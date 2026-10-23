// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/access/Ownable2Step.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/cryptography/EIP712.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

/**
 * @title AeroLedgerEscrow
 * @notice Marketplace escrow binding a **buyer**, a **registered seller** and the
 *         **platform treasury**. A buyer funds an order with native coin (ETH/BNB)
 *         or a supported ERC20; on completion the seller receives the net proceeds
 *         at the payout address governance registered for them, and the treasury
 *         receives the quoted platform fee.
 *
 * @dev ## Parties and trust model
 *
 *      | Party    | Identity                       | Powers                                          |
 *      |----------|--------------------------------|-------------------------------------------------|
 *      | Buyer    | `Order.buyer`                  | Funds escrow; cancels while Pending; timeout exit|
 *      | Seller   | `Merchant.authority`           | Confirms, completes, refunds their own orders    |
 *      | Payout   | `Order.payoutWallet` (snapshot)| Sole destination of seller proceeds              |
 *      | Treasury | `Order.treasury` (snapshot)    | Sole destination of the platform fee             |
 *      | Owner    | `Ownable2Step`                 | Governance: merchants, tokens, signer, pause     |
 *      | Quote    | `quoteSigner`                  | EIP-712 key that authorises each order's price   |
 *
 *      The earlier version released funds to `owner()` and let the buyer choose
 *      `msg.value` freely, so (a) the escrow modelled a buyer/platform relationship
 *      with no seller, and (b) a buyer could pay one wei for a $5,000 booking by
 *      calling the contract directly. Both are fixed here:
 *
 *      - Every order names a seller that governance registered in advance, and the
 *        payout address is **snapshotted into the order at creation**. No caller can
 *        pass a different destination at settlement time, and rotating a merchant's
 *        payout wallet does not retarget money already in escrow.
 *      - Every order carries an EIP-712 `Quote` signed by `quoteSigner`, binding
 *        (orderHash, buyer, seller, token, amount, fee, expiry). The contract
 *        enforces the signed amount exactly, so price is set by the backend and
 *        cannot be edited in the browser or in the calldata.
 *
 *      What the owner CANNOT do: choose a payout or refund destination, change the
 *      amount of a live order, confirm on a seller's behalf, or trap a buyer's funds
 *      (see {timeoutRefund}). What the owner CAN do, and the residual risk it
 *      carries, is documented in SECURITY_AUDIT.md.
 *
 *      ## Asset-handling properties
 *      - Native and ERC20 escrow are strictly segregated ({createOrder} is
 *        native-only, {createOrderERC20} is ERC20-only) and per-asset escrow
 *        balances are tracked, so a payout can never exceed what was deposited for
 *        that asset.
 *      - All ERC20 movements use SafeERC20, so non-standard tokens (USDT returns no
 *        bool) work and silent-false transfers revert.
 *      - The contract measures the **actual** received balance delta and requires it
 *        to be at least the quoted amount. A fee-on-transfer token therefore reverts
 *        rather than under-funding an order: see SECURITY_AUDIT.md (EVM-07).
 *      - Native transfers use a checked low-level call, so a multisig payout address
 *        cannot be bricked by the 2300-gas stipend.
 *      - Checks-Effects-Interactions plus nonReentrant on every value-moving path.
 */
contract AeroLedgerEscrow is ReentrancyGuard, Pausable, Ownable2Step, EIP712 {
    using SafeERC20 for IERC20;

    enum OrderStatus { None, Pending, Confirmed, Completed, Refunded }

    struct Order {
        address buyer;
        address seller;        // Merchant authority this order is with
        address payoutWallet;  // Snapshot of merchant payout address at creation
        address treasury;      // Snapshot of platform treasury at creation
        address token;         // address(0) for native (ETH/BNB), else the ERC20
        uint256 amount;        // Escrowed amount, equal to the signed quote amount
        uint256 fee;           // Platform fee, carved out of `amount` at payout
        OrderStatus status;
        uint256 createdAt;
        uint256 refundDeadline;
        string orderId;        // Backend order ID
    }

    struct Merchant {
        address payoutWallet;
        bool active;
    }

    /// @notice A server-issued price commitment plus the signature that authorises it.
    /// @dev Passed as a single calldata struct rather than seven loose arguments: it
    ///      keeps the signing payload and the call payload obviously identical, and
    ///      keeps the creation functions within the EVM's stack limits.
    struct QuoteInput {
        string orderId;
        address seller;
        /// address(0) for native, otherwise the ERC20 being escrowed.
        address token;
        uint256 amount;
        uint256 fee;
        uint256 expiry;
        bytes signature;
    }

    /// @dev Sentinel used internally to represent the native asset in the ledger.
    address private constant NATIVE = address(0);

    bytes32 private constant QUOTE_TYPEHASH = keccak256(
        "Quote(bytes32 orderHash,address buyer,address seller,address token,uint256 amount,uint256 fee,uint256 expiry)"
    );

    /// @notice Longest a signed quote may remain usable, bounding the damage of a leak.
    uint256 public constant MAX_QUOTE_LIFETIME = 15 minutes;

    uint256 public constant MIN_REFUND_WINDOW = 1 hours;
    uint256 public constant MAX_REFUND_WINDOW = 30 days;

    mapping(bytes32 => Order) private _orders;
    mapping(address => bool) public supportedTokens;
    mapping(address => Merchant) public merchants;

    /// @notice Total funds currently held in escrow per asset (native under address(0)).
    /// @dev Invariant: this contract's balance of each asset >= escrowBalance[asset].
    mapping(address => uint256) public escrowBalance;

    /// @notice Key whose EIP-712 signature authorises an order's price. Hot key,
    ///         deliberately separate from `owner` so it can be rotated cheaply.
    address public quoteSigner;

    /// @notice Where platform fees are sent. Snapshotted into each order at creation.
    address public treasury;

    /// @notice How long an order stays refundable, and when a buyer's timeout exit opens.
    uint256 public refundWindow;

    event OrderCreated(
        bytes32 indexed orderHash,
        address indexed buyer,
        address indexed seller,
        address token,
        uint256 amount,
        uint256 fee,
        string orderId
    );
    event OrderConfirmed(bytes32 indexed orderHash, address indexed seller);
    event OrderCompleted(
        bytes32 indexed orderHash,
        address indexed payoutWallet,
        uint256 sellerAmount,
        uint256 feeAmount
    );
    event OrderRefunded(bytes32 indexed orderHash, address indexed buyer, uint256 amount, bool timedOut);
    event TokenSupportUpdated(address indexed token, bool supported);
    event MerchantUpdated(address indexed seller, address payoutWallet, bool active);
    event QuoteSignerUpdated(address indexed previous, address indexed current);
    event TreasuryUpdated(address indexed previous, address indexed current);
    event RefundWindowUpdated(uint256 previous, uint256 current);

    error ZeroAddress();
    error InvalidRefundWindow();
    error NotNative();
    error NotERC20();
    error TokenNotSupported();
    error MerchantNotActive();
    error OrderExists();
    error InvalidStatus();
    error NotAuthorized();
    error AmountMismatch();
    error InvalidFee();
    error QuoteExpired();
    error QuoteLifetimeTooLong();
    error InvalidQuoteSignature();
    error RefundWindowExpired();
    error TimeoutNotReached();
    error EscrowUnderflow();
    error NativeTransferFailed();

    constructor(address initialOwner, address initialQuoteSigner, address initialTreasury, uint256 initialRefundWindow)
        Ownable(initialOwner)
        EIP712("AeroLedgerEscrow", "2")
    {
        if (initialQuoteSigner == address(0) || initialTreasury == address(0)) revert ZeroAddress();
        if (initialRefundWindow < MIN_REFUND_WINDOW || initialRefundWindow > MAX_REFUND_WINDOW) {
            revert InvalidRefundWindow();
        }

        quoteSigner = initialQuoteSigner;
        treasury = initialTreasury;
        refundWindow = initialRefundWindow;

        // Native coin is always escrowable and is not an ERC20.
        supportedTokens[NATIVE] = true;

        emit TokenSupportUpdated(NATIVE, true);
        emit QuoteSignerUpdated(address(0), initialQuoteSigner);
        emit TreasuryUpdated(address(0), initialTreasury);
        emit RefundWindowUpdated(0, initialRefundWindow);
    }

    // ---------------------------------------------------------------------
    // Governance
    // ---------------------------------------------------------------------

    function setQuoteSigner(address signer) external onlyOwner {
        if (signer == address(0)) revert ZeroAddress();
        emit QuoteSignerUpdated(quoteSigner, signer);
        quoteSigner = signer;
    }

    /// @dev Only affects orders created after this call; live orders keep their snapshot.
    function setTreasury(address newTreasury) external onlyOwner {
        if (newTreasury == address(0)) revert ZeroAddress();
        emit TreasuryUpdated(treasury, newTreasury);
        treasury = newTreasury;
    }

    function setRefundWindow(uint256 window) external onlyOwner {
        if (window < MIN_REFUND_WINDOW || window > MAX_REFUND_WINDOW) revert InvalidRefundWindow();
        emit RefundWindowUpdated(refundWindow, window);
        refundWindow = window;
    }

    /// @notice Pause NEW order creation. Settlement, refunds and {timeoutRefund} stay
    ///         live, so a pause can never strand escrowed funds.
    function pause() external onlyOwner { _pause(); }
    function unpause() external onlyOwner { _unpause(); }

    /// @notice Register or update a seller and the address their proceeds must go to.
    function setMerchant(address seller, address payoutWallet, bool active) external onlyOwner {
        if (seller == address(0) || payoutWallet == address(0)) revert ZeroAddress();
        merchants[seller] = Merchant({ payoutWallet: payoutWallet, active: active });
        emit MerchantUpdated(seller, payoutWallet, active);
    }

    function addSupportedToken(address token) external onlyOwner {
        if (token == NATIVE) revert NotERC20();
        supportedTokens[token] = true;
        emit TokenSupportUpdated(token, true);
    }

    /// @notice De-register an ERC20. Existing orders in that token stay refundable and
    ///         completable; this only blocks new orders.
    function removeSupportedToken(address token) external onlyOwner {
        if (token == NATIVE) revert NotERC20();
        supportedTokens[token] = false;
        emit TokenSupportUpdated(token, false);
    }

    // ---------------------------------------------------------------------
    // Order creation
    // ---------------------------------------------------------------------

    /**
     * @notice Create a native-coin escrow order against a signed quote.
     * @dev `msg.value` must equal the signed `amount` exactly -- the price is the
     *      backend's to set, not the caller's.
     */
    function createOrder(QuoteInput calldata q) external payable nonReentrant whenNotPaused {
        if (q.token != NATIVE) revert NotNative();
        if (msg.value != q.amount) revert AmountMismatch();

        bytes32 orderHash = _openOrder(q);
        escrowBalance[NATIVE] += q.amount;

        emit OrderCreated(orderHash, msg.sender, q.seller, NATIVE, q.amount, q.fee, q.orderId);
    }

    /**
     * @notice Create an ERC20 escrow order against a signed quote.
     * @dev Credits the measured balance delta and requires it to be at least the
     *      quoted amount, so a fee-on-transfer token reverts rather than silently
     *      under-funding the seller. Surplus (a rebase up mid-call) stays with the
     *      order and settles to the treasury.
     */
    function createOrderERC20(QuoteInput calldata q) external nonReentrant whenNotPaused {
        if (q.token == NATIVE) revert NotERC20();
        if (!supportedTokens[q.token]) revert TokenNotSupported();

        bytes32 orderHash = _openOrder(q);

        // Keep the real received figure so the escrow ledger stays exact.
        uint256 received = _pullTokens(q.token, q.amount);
        _orders[orderHash].amount = received;
        escrowBalance[q.token] += received;

        emit OrderCreated(orderHash, msg.sender, q.seller, q.token, received, q.fee, q.orderId);
    }

    /// @dev Pull `amount` of `token` from the caller and return the balance delta the
    ///      contract actually gained. Reverts if the delta is short of `amount`, which
    ///      is what makes a fee-on-transfer token unusable here rather than silently
    ///      under-funding the seller against a signed quote.
    function _pullTokens(address token, uint256 amount) private returns (uint256 received) {
        IERC20 erc20 = IERC20(token);
        uint256 balBefore = erc20.balanceOf(address(this));
        erc20.safeTransferFrom(msg.sender, address(this), amount);
        received = erc20.balanceOf(address(this)) - balBefore;
        if (received < amount) revert AmountMismatch();
    }

    /// @dev Shared validation + storage write for both creation paths.
    function _openOrder(QuoteInput calldata q) private returns (bytes32 orderHash) {
        if (q.amount == 0) revert AmountMismatch();
        if (q.fee >= q.amount) revert InvalidFee();
        if (block.timestamp > q.expiry) revert QuoteExpired();
        if (q.expiry - block.timestamp > MAX_QUOTE_LIFETIME) revert QuoteLifetimeTooLong();

        Merchant memory merchant = merchants[q.seller];
        if (!merchant.active) revert MerchantNotActive();

        orderHash = _computeOrderHash(msg.sender, q.orderId);
        // The order slot itself is the replay guard: a signed quote can be used at
        // most once, because the second attempt collides with an existing order.
        if (_orders[orderHash].status != OrderStatus.None) revert OrderExists();

        _verifyQuote(orderHash, q);

        _orders[orderHash] = Order({
            buyer: msg.sender,
            seller: q.seller,
            payoutWallet: merchant.payoutWallet,
            treasury: treasury,
            token: q.token,
            amount: q.amount,
            fee: q.fee,
            status: OrderStatus.Pending,
            createdAt: block.timestamp,
            refundDeadline: block.timestamp + refundWindow,
            orderId: q.orderId
        });
    }

    function _verifyQuote(bytes32 orderHash, QuoteInput calldata q) private view {
        bytes32 digest = _hashTypedDataV4(
            keccak256(
                abi.encode(QUOTE_TYPEHASH, orderHash, msg.sender, q.seller, q.token, q.amount, q.fee, q.expiry)
            )
        );
        (address recovered, ECDSA.RecoverError err, ) = ECDSA.tryRecover(digest, q.signature);
        if (err != ECDSA.RecoverError.NoError || recovered != quoteSigner) {
            revert InvalidQuoteSignature();
        }
    }

    // ---------------------------------------------------------------------
    // Order lifecycle
    // ---------------------------------------------------------------------

    /// @notice The seller accepts the order: Pending -> Confirmed.
    /// @dev Only the seller may do this. The owner deliberately cannot confirm on a
    ///      seller's behalf — confirmation is the seller's commitment to fulfil.
    function confirmOrder(bytes32 orderHash) external {
        Order storage order = _orders[orderHash];
        if (order.status != OrderStatus.Pending) revert InvalidStatus();
        if (msg.sender != order.seller) revert NotAuthorized();

        order.status = OrderStatus.Confirmed;
        emit OrderConfirmed(orderHash, order.seller);
    }

    /// @notice Release a confirmed order: net proceeds to the seller's registered
    ///         payout address, fee to the treasury snapshotted on the order.
    function completeOrder(bytes32 orderHash) external nonReentrant {
        Order storage order = _orders[orderHash];
        if (order.status != OrderStatus.Confirmed) revert InvalidStatus();
        if (msg.sender != order.seller && msg.sender != owner()) revert NotAuthorized();

        uint256 amount = order.amount;
        uint256 feeAmount = order.fee;
        uint256 sellerAmount = amount - feeAmount; // fee < amount enforced at creation
        address token = order.token;
        address payoutWallet = order.payoutWallet;
        address orderTreasury = order.treasury;

        // Effects before interactions.
        order.status = OrderStatus.Completed;
        _debitEscrow(token, amount);

        if (sellerAmount > 0) _payout(token, payoutWallet, sellerAmount);
        if (feeAmount > 0) _payout(token, orderTreasury, feeAmount);

        emit OrderCompleted(orderHash, payoutWallet, sellerAmount, feeAmount);
    }

    /**
     * @notice Return escrowed funds to the recorded buyer, within the refund window.
     * @dev Authorisation by status:
     *      - `Pending`:   buyer, seller or owner (the seller has not accepted yet),
     *      - `Confirmed`: seller or owner only (the buyer uses {timeoutRefund}).
     */
    function refundOrder(bytes32 orderHash) external nonReentrant {
        Order storage order = _orders[orderHash];
        OrderStatus status = order.status;

        if (status == OrderStatus.Pending) {
            if (msg.sender != order.buyer && msg.sender != order.seller && msg.sender != owner()) {
                revert NotAuthorized();
            }
        } else if (status == OrderStatus.Confirmed) {
            if (msg.sender != order.seller && msg.sender != owner()) revert NotAuthorized();
        } else {
            revert InvalidStatus();
        }

        if (block.timestamp > order.refundDeadline) revert RefundWindowExpired();
        _settleRefund(orderHash, order, false);
    }

    /**
     * @notice Unilateral buyer exit. If the seller never confirms, or confirms and
     *         never completes, the buyer reclaims their funds once the refund
     *         deadline passes. Closes the "funds held hostage by inaction" gap.
     */
    function timeoutRefund(bytes32 orderHash) external nonReentrant {
        Order storage order = _orders[orderHash];
        if (order.status != OrderStatus.Pending && order.status != OrderStatus.Confirmed) {
            revert InvalidStatus();
        }
        if (msg.sender != order.buyer) revert NotAuthorized();
        if (block.timestamp <= order.refundDeadline) revert TimeoutNotReached();

        _settleRefund(orderHash, order, true);
    }

    function _settleRefund(bytes32 orderHash, Order storage order, bool timedOut) private {
        uint256 amount = order.amount;
        address token = order.token;
        address buyer = order.buyer;

        order.status = OrderStatus.Refunded;
        _debitEscrow(token, amount);

        _payout(token, buyer, amount);
        emit OrderRefunded(orderHash, buyer, amount, timedOut);
    }

    // ---------------------------------------------------------------------
    // Views
    // ---------------------------------------------------------------------

    function getOrder(bytes32 orderHash) external view returns (Order memory) {
        return _orders[orderHash];
    }

    function orders(bytes32 orderHash) external view returns (Order memory) {
        return _orders[orderHash];
    }

    /// @notice Deterministically compute the storage hash for a (buyer, orderId) pair.
    function computeOrderHash(address buyer, string calldata orderId) external pure returns (bytes32) {
        return _computeOrderHash(buyer, orderId);
    }

    /// @notice EIP-712 domain separator, exposed so the backend can sanity-check that
    ///         it is signing for the chain and contract it thinks it is.
    function domainSeparator() external view returns (bytes32) {
        return _domainSeparatorV4();
    }

    /// @notice The digest a quote signature must cover. Mirrors {_verifyQuote}.
    function quoteDigest(
        bytes32 orderHash,
        address buyer,
        address seller,
        address token,
        uint256 amount,
        uint256 fee,
        uint256 expiry
    ) external view returns (bytes32) {
        return _hashTypedDataV4(
            keccak256(abi.encode(QUOTE_TYPEHASH, orderHash, buyer, seller, token, amount, fee, expiry))
        );
    }

    /// @notice Funds held by this contract above what the escrow ledger accounts for,
    ///         i.e. tokens or coin sent here by mistake.
    function unaccountedBalance(address token) public view returns (uint256) {
        uint256 held = token == NATIVE ? address(this).balance : IERC20(token).balanceOf(address(this));
        uint256 owed = escrowBalance[token];
        return held > owed ? held - owed : 0;
    }

    /// @notice Recover assets accidentally sent to this contract. Cannot touch escrowed
    ///         funds: the amount is capped by {unaccountedBalance}, which subtracts
    ///         every live order's deposit.
    function sweepUnaccounted(address token, address to) external onlyOwner nonReentrant {
        if (to == address(0)) revert ZeroAddress();
        uint256 amount = unaccountedBalance(token);
        if (amount == 0) revert EscrowUnderflow();
        _payout(token, to, amount);
    }

    // ---------------------------------------------------------------------
    // Internal helpers
    // ---------------------------------------------------------------------

    /// @dev Order identity is (buyer, orderId): a given buyer can have one live order
    ///      per backend orderId. Deliberately excludes `block.timestamp`, which the
    ///      original design mixed in and which made the hash unpredictable off-chain.
    function _computeOrderHash(address buyer, string calldata orderId) private pure returns (bytes32) {
        return keccak256(abi.encode(buyer, orderId));
    }

    function _debitEscrow(address token, uint256 amount) private {
        uint256 bal = escrowBalance[token];
        if (bal < amount) revert EscrowUnderflow();
        escrowBalance[token] = bal - amount;
    }

    function _payout(address token, address to, uint256 amount) private {
        if (token == NATIVE) {
            (bool ok, ) = payable(to).call{value: amount}("");
            if (!ok) revert NativeTransferFailed();
        } else {
            IERC20(token).safeTransfer(to, amount);
        }
    }
}
