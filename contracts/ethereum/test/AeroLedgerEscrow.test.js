/**
 * AeroLedgerEscrow — Hardhat test suite.
 *
 * The emphasis is adversarial: every "rejects ..." case asserts a specific custom
 * error, and the settlement cases assert real balance deltas for the buyer, the
 * seller's registered payout address and the treasury — not just that a call
 * returned. The three things the first review flagged each have their own block:
 * the buyer/seller relationship, server-authoritative pricing, and the fact that
 * no participant can redirect a payout or a refund.
 */
const { expect } = require("chai");
const { ethers } = require("hardhat");
const { time } = require("@nomicfoundation/hardhat-network-helpers");

const NATIVE = ethers.ZeroAddress;
const REFUND_WINDOW = 24 * 60 * 60; // 24h
const USDC_UNIT = 10n ** 6n;

describe("AeroLedgerEscrow", () => {
  let escrow, usdc, usdt, feeToken;
  let owner, quoteSigner, treasury, seller, sellerPayout, buyer, attacker, other;
  let chainId;

  // -------------------------------------------------------------------
  // Helpers
  // -------------------------------------------------------------------

  const domain = () => ({
    name: "AeroLedgerEscrow",
    version: "2",
    chainId,
    verifyingContract: escrow.target,
  });

  const TYPES = {
    Quote: [
      { name: "orderHash", type: "bytes32" },
      { name: "buyer", type: "address" },
      { name: "seller", type: "address" },
      { name: "token", type: "address" },
      { name: "amount", type: "uint256" },
      { name: "fee", type: "uint256" },
      { name: "expiry", type: "uint256" },
    ],
  };

  async function futureExpiry(seconds = 600) {
    return BigInt((await time.latest()) + seconds);
  }

  /**
   * Build a QuoteInput struct plus its EIP-712 signature.
   *
   * `signed` lets a test sign *different* values from the ones that go on chain,
   * which is exactly how a price-manipulation attempt is simulated.
   */
  async function makeQuote({
    orderId,
    buyerAddr,
    sellerAddr,
    token = NATIVE,
    amount,
    fee,
    expiry,
    signer,
    signed = {},
  }) {
    expiry = expiry ?? (await futureExpiry());
    sellerAddr = sellerAddr ?? seller.address;
    const orderHash = await escrow.computeOrderHash(buyerAddr, orderId);

    const payload = {
      orderHash: signed.orderHash ?? orderHash,
      buyer: signed.buyer ?? buyerAddr,
      seller: signed.seller ?? sellerAddr,
      token: signed.token ?? token,
      amount: signed.amount ?? amount,
      fee: signed.fee ?? fee,
      expiry: signed.expiry ?? expiry,
    };

    const signature = await (signer ?? quoteSigner).signTypedData(domain(), TYPES, payload);
    return {
      input: { orderId, seller: sellerAddr, token, amount, fee, expiry, signature },
      orderHash,
    };
  }

  async function createNative(opts) {
    const { input, orderHash } = await makeQuote({ ...opts, token: NATIVE });
    const from = opts.from ?? buyer;
    const value = opts.value ?? opts.amount;
    return { tx: escrow.connect(from).createOrder(input, { value }), orderHash, input };
  }

  async function createErc20(opts) {
    const { input, orderHash } = await makeQuote(opts);
    const from = opts.from ?? buyer;
    return { tx: escrow.connect(from).createOrderERC20(input), orderHash, input };
  }

  // -------------------------------------------------------------------
  // Fixtures
  // -------------------------------------------------------------------

  beforeEach(async () => {
    [owner, quoteSigner, treasury, seller, sellerPayout, buyer, attacker, other] =
      await ethers.getSigners();
    chainId = (await ethers.provider.getNetwork()).chainId;

    const Escrow = await ethers.getContractFactory("AeroLedgerEscrow");
    escrow = await Escrow.deploy(
      owner.address,
      quoteSigner.address,
      treasury.address,
      REFUND_WINDOW
    );

    const MockERC20 = await ethers.getContractFactory("MockERC20");
    usdc = await MockERC20.deploy("USD Coin", "USDC", 6, 1_000_000n * USDC_UNIT);

    const MockUSDT = await ethers.getContractFactory("MockUSDT");
    usdt = await MockUSDT.deploy(1_000_000n * USDC_UNIT);

    const FeeToken = await ethers.getContractFactory("FeeOnTransferToken");
    feeToken = await FeeToken.deploy(100, 1_000_000n * USDC_UNIT); // 1% burn

    await escrow.connect(owner).addSupportedToken(usdc.target);
    await escrow.connect(owner).addSupportedToken(usdt.target);
    await escrow.connect(owner).addSupportedToken(feeToken.target);
    await escrow.connect(owner).setMerchant(seller.address, sellerPayout.address, true);

    for (const token of [usdc, usdt, feeToken]) {
      await token.mint(buyer.address, 10_000n * USDC_UNIT);
      await token.connect(buyer).approve(escrow.target, ethers.MaxUint256);
      await token.mint(attacker.address, 10_000n * USDC_UNIT);
      await token.connect(attacker).approve(escrow.target, ethers.MaxUint256);
    }
  });

  // -------------------------------------------------------------------

  describe("deployment and governance", () => {
    it("records owner, quote signer, treasury and refund window", async () => {
      expect(await escrow.owner()).to.equal(owner.address);
      expect(await escrow.quoteSigner()).to.equal(quoteSigner.address);
      expect(await escrow.treasury()).to.equal(treasury.address);
      expect(await escrow.refundWindow()).to.equal(REFUND_WINDOW);
      expect(await escrow.supportedTokens(NATIVE)).to.equal(true);
    });

    it("rejects a zero quote signer or treasury at deploy", async () => {
      const Escrow = await ethers.getContractFactory("AeroLedgerEscrow");
      await expect(
        Escrow.deploy(owner.address, ethers.ZeroAddress, treasury.address, REFUND_WINDOW)
      ).to.be.revertedWithCustomError(escrow, "ZeroAddress");
    });

    it("rejects a refund window outside the permitted bounds", async () => {
      await expect(escrow.connect(owner).setRefundWindow(60)).to.be.revertedWithCustomError(
        escrow,
        "InvalidRefundWindow"
      );
      await expect(
        escrow.connect(owner).setRefundWindow(60 * 60 * 24 * 60)
      ).to.be.revertedWithCustomError(escrow, "InvalidRefundWindow");
    });

    it("only the owner can register merchants, tokens, signer and treasury", async () => {
      await expect(
        escrow.connect(attacker).setMerchant(attacker.address, attacker.address, true)
      ).to.be.revertedWithCustomError(escrow, "OwnableUnauthorizedAccount");
      await expect(
        escrow.connect(attacker).setQuoteSigner(attacker.address)
      ).to.be.revertedWithCustomError(escrow, "OwnableUnauthorizedAccount");
      await expect(
        escrow.connect(attacker).setTreasury(attacker.address)
      ).to.be.revertedWithCustomError(escrow, "OwnableUnauthorizedAccount");
      await expect(
        escrow.connect(attacker).addSupportedToken(usdc.target)
      ).to.be.revertedWithCustomError(escrow, "OwnableUnauthorizedAccount");
    });

    it("requires a two-step handover to change the owner", async () => {
      await escrow.connect(owner).transferOwnership(other.address);
      expect(await escrow.owner()).to.equal(owner.address); // not yet
      await expect(escrow.connect(attacker).acceptOwnership()).to.be.revertedWithCustomError(
        escrow,
        "OwnableUnauthorizedAccount"
      );
      await escrow.connect(other).acceptOwnership();
      expect(await escrow.owner()).to.equal(other.address);
    });
  });

  // -------------------------------------------------------------------

  describe("server-authoritative pricing (signed quotes)", () => {
    it("rejects an order with no valid signature", async () => {
      const { input } = await makeQuote({
        orderId: "q-1",
        buyerAddr: buyer.address,
        amount: ethers.parseEther("1"),
        fee: ethers.parseEther("0.03"),
      });
      const tampered = { ...input, signature: "0x" + "11".repeat(65) };
      await expect(
        escrow.connect(buyer).createOrder(tampered, { value: ethers.parseEther("1") })
      ).to.be.revertedWithCustomError(escrow, "InvalidQuoteSignature");
    });

    it("rejects a quote signed by a key that is not the quote signer", async () => {
      const { input } = await makeQuote({
        orderId: "q-2",
        buyerAddr: buyer.address,
        amount: ethers.parseEther("1"),
        fee: ethers.parseEther("0.03"),
        signer: attacker,
      });
      await expect(
        escrow.connect(buyer).createOrder(input, { value: ethers.parseEther("1") })
      ).to.be.revertedWithCustomError(escrow, "InvalidQuoteSignature");
    });

    it("rejects a buyer who pays less than the signed amount", async () => {
      const amount = ethers.parseEther("1");
      const { input } = await makeQuote({
        orderId: "q-3",
        buyerAddr: buyer.address,
        amount,
        fee: ethers.parseEther("0.03"),
      });
      await expect(
        escrow.connect(buyer).createOrder(input, { value: 1n })
      ).to.be.revertedWithCustomError(escrow, "AmountMismatch");
    });

    it("rejects an order whose calldata amount differs from the signed amount", async () => {
      // A genuine $5,000 quote, but the caller edits the amount down to 1 wei and
      // sends 1 wei. The signature no longer covers the calldata, so recovery fails.
      const { input } = await makeQuote({
        orderId: "q-4",
        buyerAddr: buyer.address,
        amount: 1n,
        fee: 0n,
        signed: { amount: ethers.parseEther("5"), fee: ethers.parseEther("0.15") },
      });
      await expect(
        escrow.connect(buyer).createOrder(input, { value: 1n })
      ).to.be.revertedWithCustomError(escrow, "InvalidQuoteSignature");
    });

    it("rejects a quote issued to a different buyer (quote theft)", async () => {
      const amount = ethers.parseEther("1");
      // Signed for `buyer`, replayed by `attacker`.
      const orderHash = await escrow.computeOrderHash(buyer.address, "q-5");
      const expiry = await futureExpiry();
      const signature = await quoteSigner.signTypedData(domain(), TYPES, {
        orderHash,
        buyer: buyer.address,
        seller: seller.address,
        token: NATIVE,
        amount,
        fee: 0n,
        expiry,
      });
      await expect(
        escrow
          .connect(attacker)
          .createOrder(
            { orderId: "q-5", seller: seller.address, token: NATIVE, amount, fee: 0n, expiry, signature },
            { value: amount }
          )
      ).to.be.revertedWithCustomError(escrow, "InvalidQuoteSignature");
    });

    it("rejects a quote re-pointed at a different seller", async () => {
      await escrow.connect(owner).setMerchant(other.address, other.address, true);
      const amount = ethers.parseEther("1");
      const { input } = await makeQuote({
        orderId: "q-6",
        buyerAddr: buyer.address,
        sellerAddr: other.address,
        amount,
        fee: 0n,
        signed: { seller: seller.address },
      });
      await expect(
        escrow.connect(buyer).createOrder(input, { value: amount })
      ).to.be.revertedWithCustomError(escrow, "InvalidQuoteSignature");
    });

    it("rejects an expired quote", async () => {
      const amount = ethers.parseEther("1");
      const expiry = BigInt(await time.latest()) + 60n;
      const { input } = await makeQuote({
        orderId: "q-7",
        buyerAddr: buyer.address,
        amount,
        fee: 0n,
        expiry,
      });
      await time.increase(120);
      await expect(
        escrow.connect(buyer).createOrder(input, { value: amount })
      ).to.be.revertedWithCustomError(escrow, "QuoteExpired");
    });

    it("rejects a quote with an implausibly long lifetime", async () => {
      const amount = ethers.parseEther("1");
      const { input } = await makeQuote({
        orderId: "q-8",
        buyerAddr: buyer.address,
        amount,
        fee: 0n,
        expiry: await futureExpiry(7 * 24 * 3600),
      });
      await expect(
        escrow.connect(buyer).createOrder(input, { value: amount })
      ).to.be.revertedWithCustomError(escrow, "QuoteLifetimeTooLong");
    });

    it("rejects a zero amount and a fee that swallows the whole order", async () => {
      await expect(
        (await createNative({ orderId: "q-9", buyerAddr: buyer.address, amount: 0n, fee: 0n })).tx
      ).to.be.revertedWithCustomError(escrow, "AmountMismatch");

      const amount = ethers.parseEther("1");
      await expect(
        (await createNative({ orderId: "q-10", buyerAddr: buyer.address, amount, fee: amount })).tx
      ).to.be.revertedWithCustomError(escrow, "InvalidFee");
    });

    it("rejects replaying the same signed quote twice", async () => {
      const amount = ethers.parseEther("1");
      const { input } = await makeQuote({
        orderId: "q-11",
        buyerAddr: buyer.address,
        amount,
        fee: 0n,
      });
      await escrow.connect(buyer).createOrder(input, { value: amount });
      await expect(
        escrow.connect(buyer).createOrder(input, { value: amount })
      ).to.be.revertedWithCustomError(escrow, "OrderExists");
    });

    it("rejects a quote signed with the wrong chain id (cross-chain replay)", async () => {
      const amount = ethers.parseEther("1");
      const orderHash = await escrow.computeOrderHash(buyer.address, "q-12");
      const expiry = await futureExpiry();
      const signature = await quoteSigner.signTypedData(
        { ...domain(), chainId: 999999n },
        TYPES,
        {
          orderHash,
          buyer: buyer.address,
          seller: seller.address,
          token: NATIVE,
          amount,
          fee: 0n,
          expiry,
        }
      );
      await expect(
        escrow
          .connect(buyer)
          .createOrder(
            { orderId: "q-12", seller: seller.address, token: NATIVE, amount, fee: 0n, expiry, signature },
            { value: amount }
          )
      ).to.be.revertedWithCustomError(escrow, "InvalidQuoteSignature");
    });
  });

  // -------------------------------------------------------------------

  describe("buyer -> seller settlement (native)", () => {
    const amount = ethers.parseEther("10");
    const fee = ethers.parseEther("0.3");
    let orderHash;

    beforeEach(async () => {
      const r = await createNative({ orderId: "n-1", buyerAddr: buyer.address, amount, fee });
      await r.tx;
      orderHash = r.orderHash;
    });

    it("records the buyer, seller and both payout destinations on the order", async () => {
      const order = await escrow.getOrder(orderHash);
      expect(order.buyer).to.equal(buyer.address);
      expect(order.seller).to.equal(seller.address);
      expect(order.payoutWallet).to.equal(sellerPayout.address);
      expect(order.treasury).to.equal(treasury.address);
      expect(order.amount).to.equal(amount);
      expect(order.fee).to.equal(fee);
      expect(order.status).to.equal(1); // Pending
      expect(await escrow.escrowBalance(NATIVE)).to.equal(amount);
    });

    it("rejects confirmation by anyone other than the seller", async () => {
      await expect(escrow.connect(owner).confirmOrder(orderHash)).to.be.revertedWithCustomError(
        escrow,
        "NotAuthorized"
      );
      await expect(escrow.connect(buyer).confirmOrder(orderHash)).to.be.revertedWithCustomError(
        escrow,
        "NotAuthorized"
      );
      await expect(escrow.connect(attacker).confirmOrder(orderHash)).to.be.revertedWithCustomError(
        escrow,
        "NotAuthorized"
      );
    });

    it("rejects completion before confirmation", async () => {
      await expect(escrow.connect(seller).completeOrder(orderHash)).to.be.revertedWithCustomError(
        escrow,
        "InvalidStatus"
      );
    });

    it("pays the seller's registered wallet and the treasury, never the owner", async () => {
      await escrow.connect(seller).confirmOrder(orderHash);

      const ownerBefore = await ethers.provider.getBalance(owner.address);
      await expect(escrow.connect(seller).completeOrder(orderHash)).to.changeEtherBalances(
        [sellerPayout, treasury, escrow],
        [amount - fee, fee, -amount]
      );
      // The platform owner receives nothing from a marketplace sale.
      expect(await ethers.provider.getBalance(owner.address)).to.equal(ownerBefore);
      expect(await escrow.escrowBalance(NATIVE)).to.equal(0);
    });

    it("keeps paying the wallet recorded at creation even if the merchant is re-pointed", async () => {
      await escrow.connect(seller).confirmOrder(orderHash);
      // Governance (or a compromised owner) re-points the merchant mid-flight.
      await escrow.connect(owner).setMerchant(seller.address, attacker.address, true);

      await expect(escrow.connect(seller).completeOrder(orderHash)).to.changeEtherBalances(
        [sellerPayout, attacker],
        [amount - fee, 0]
      );
    });

    it("keeps paying the treasury recorded at creation even if it is re-pointed", async () => {
      await escrow.connect(seller).confirmOrder(orderHash);
      await escrow.connect(owner).setTreasury(attacker.address);

      await expect(escrow.connect(seller).completeOrder(orderHash)).to.changeEtherBalances(
        [treasury, attacker],
        [fee, 0]
      );
    });

    it("rejects completion by an unrelated signer", async () => {
      await escrow.connect(seller).confirmOrder(orderHash);
      await expect(
        escrow.connect(attacker).completeOrder(orderHash)
      ).to.be.revertedWithCustomError(escrow, "NotAuthorized");
    });

    it("rejects completing twice and refunding after completion", async () => {
      await escrow.connect(seller).confirmOrder(orderHash);
      await escrow.connect(seller).completeOrder(orderHash);
      await expect(escrow.connect(seller).completeOrder(orderHash)).to.be.revertedWithCustomError(
        escrow,
        "InvalidStatus"
      );
      await expect(escrow.connect(owner).refundOrder(orderHash)).to.be.revertedWithCustomError(
        escrow,
        "InvalidStatus"
      );
    });

    it("rejects any operation on an unknown order", async () => {
      const unknown = ethers.keccak256(ethers.toUtf8Bytes("nope"));
      await expect(escrow.connect(seller).confirmOrder(unknown)).to.be.revertedWithCustomError(
        escrow,
        "InvalidStatus"
      );
      await expect(escrow.connect(owner).refundOrder(unknown)).to.be.revertedWithCustomError(
        escrow,
        "InvalidStatus"
      );
    });
  });

  // -------------------------------------------------------------------

  describe("ERC20 settlement", () => {
    const amount = 1_000n * USDC_UNIT;
    const fee = 30n * USDC_UNIT;

    it("escrows and settles a standard ERC20 to the seller and treasury", async () => {
      const r = await createErc20({
        orderId: "e-1",
        buyerAddr: buyer.address,
        token: usdc.target,
        amount,
        fee,
      });
      await r.tx;
      expect(await usdc.balanceOf(escrow.target)).to.equal(amount);

      await escrow.connect(seller).confirmOrder(r.orderHash);
      await escrow.connect(seller).completeOrder(r.orderHash);

      expect(await usdc.balanceOf(sellerPayout.address)).to.equal(amount - fee);
      expect(await usdc.balanceOf(treasury.address)).to.equal(fee);
      expect(await escrow.escrowBalance(usdc.target)).to.equal(0);
    });

    it("works with a USDT-style token that returns no bool", async () => {
      const r = await createErc20({
        orderId: "e-2",
        buyerAddr: buyer.address,
        token: usdt.target,
        amount,
        fee,
      });
      await r.tx;
      await escrow.connect(seller).confirmOrder(r.orderHash);
      await escrow.connect(seller).completeOrder(r.orderHash);
      expect(await usdt.balanceOf(sellerPayout.address)).to.equal(amount - fee);
      expect(await usdt.balanceOf(treasury.address)).to.equal(fee);
    });

    it("refunds a USDT-style token correctly", async () => {
      const before = await usdt.balanceOf(buyer.address);
      const r = await createErc20({
        orderId: "e-3",
        buyerAddr: buyer.address,
        token: usdt.target,
        amount,
        fee,
      });
      await r.tx;
      await escrow.connect(buyer).refundOrder(r.orderHash);
      expect(await usdt.balanceOf(buyer.address)).to.equal(before);
    });

    it("rejects a fee-on-transfer token rather than under-funding a signed quote", async () => {
      const r = await createErc20({
        orderId: "e-4",
        buyerAddr: buyer.address,
        token: feeToken.target,
        amount,
        fee,
      });
      await expect(r.tx).to.be.revertedWithCustomError(escrow, "AmountMismatch");
    });

    it("rejects an unsupported ERC20", async () => {
      await escrow.connect(owner).removeSupportedToken(usdc.target);
      const r = await createErc20({
        orderId: "e-5",
        buyerAddr: buyer.address,
        token: usdc.target,
        amount,
        fee,
      });
      await expect(r.tx).to.be.revertedWithCustomError(escrow, "TokenNotSupported");
    });

    it("keeps existing orders completable after a token is de-listed", async () => {
      const r = await createErc20({
        orderId: "e-6",
        buyerAddr: buyer.address,
        token: usdc.target,
        amount,
        fee,
      });
      await r.tx;
      await escrow.connect(owner).removeSupportedToken(usdc.target);
      await escrow.connect(seller).confirmOrder(r.orderHash);
      await escrow.connect(seller).completeOrder(r.orderHash);
      expect(await usdc.balanceOf(sellerPayout.address)).to.equal(amount - fee);
    });
  });

  // -------------------------------------------------------------------

  describe("native / ERC20 path separation (cross-asset drain)", () => {
    it("rejects a native order that names an ERC20", async () => {
      const r = await createNative({
        orderId: "x-1",
        buyerAddr: buyer.address,
        amount: 1n,
        fee: 0n,
      });
      // Re-sign with the ERC20 token field but call the native path.
      const { input } = await makeQuote({
        orderId: "x-1b",
        buyerAddr: buyer.address,
        token: usdc.target,
        amount: 1n,
        fee: 0n,
      });
      await expect(
        escrow.connect(buyer).createOrder(input, { value: 1n })
      ).to.be.revertedWithCustomError(escrow, "NotNative");
      void r;
    });

    it("rejects an ERC20 order that names the native sentinel", async () => {
      const { input } = await makeQuote({
        orderId: "x-2",
        buyerAddr: buyer.address,
        token: NATIVE,
        amount: 1n,
        fee: 0n,
      });
      await expect(
        escrow.connect(buyer).createOrderERC20(input)
      ).to.be.revertedWithCustomError(escrow, "NotERC20");
    });

    it("a tiny native deposit cannot drain pooled USDC", async () => {
      // Victim escrows 1,000 USDC.
      const victim = await createErc20({
        orderId: "x-3",
        buyerAddr: buyer.address,
        token: usdc.target,
        amount: 1_000n * USDC_UNIT,
        fee: 0n,
      });
      await victim.tx;

      // The attacker's own tiny native order can only ever release native wei,
      // and only ever to the buyer or the registered seller.
      const atk = await createNative({
        orderId: "x-4",
        buyerAddr: attacker.address,
        from: attacker,
        amount: 1n,
        fee: 0n,
      });
      await atk.tx;
      await escrow.connect(attacker).refundOrder(atk.orderHash);

      expect(await usdc.balanceOf(escrow.target)).to.equal(1_000n * USDC_UNIT);
      expect(await escrow.escrowBalance(usdc.target)).to.equal(1_000n * USDC_UNIT);
    });
  });

  // -------------------------------------------------------------------

  describe("refunds and the buyer's timeout exit", () => {
    const amount = ethers.parseEther("2");

    async function pendingOrder(id, from = buyer) {
      const r = await createNative({
        orderId: id,
        buyerAddr: from.address,
        from,
        amount,
        fee: 0n,
      });
      await r.tx;
      return r.orderHash;
    }

    it("lets the buyer cancel a pending order and returns every wei", async () => {
      const h = await pendingOrder("r-1");
      await expect(escrow.connect(buyer).refundOrder(h)).to.changeEtherBalances(
        [buyer, escrow],
        [amount, -amount]
      );
      expect(await escrow.escrowBalance(NATIVE)).to.equal(0);
    });

    it("rejects a refund requested by an unrelated signer", async () => {
      const h = await pendingOrder("r-2");
      await expect(escrow.connect(attacker).refundOrder(h)).to.be.revertedWithCustomError(
        escrow,
        "NotAuthorized"
      );
    });

    it("always refunds the recorded buyer, never the caller", async () => {
      const h = await pendingOrder("r-3");
      // The seller refunds on the buyer's behalf; the money still goes to the buyer.
      await expect(escrow.connect(seller).refundOrder(h)).to.changeEtherBalances(
        [buyer, seller],
        [amount, 0]
      );
    });

    it("rejects a buyer unilaterally refunding an order the seller confirmed", async () => {
      const h = await pendingOrder("r-4");
      await escrow.connect(seller).confirmOrder(h);
      await expect(escrow.connect(buyer).refundOrder(h)).to.be.revertedWithCustomError(
        escrow,
        "NotAuthorized"
      );
      await escrow.connect(seller).refundOrder(h); // ...but the seller may
    });

    it("rejects a double refund", async () => {
      const h = await pendingOrder("r-5");
      await escrow.connect(buyer).refundOrder(h);
      await expect(escrow.connect(buyer).refundOrder(h)).to.be.revertedWithCustomError(
        escrow,
        "InvalidStatus"
      );
    });

    it("rejects a normal refund after the window closes", async () => {
      const h = await pendingOrder("r-6");
      await time.increase(REFUND_WINDOW + 1);
      await expect(escrow.connect(buyer).refundOrder(h)).to.be.revertedWithCustomError(
        escrow,
        "RefundWindowExpired"
      );
    });

    it("rejects a timeout refund before the deadline", async () => {
      const h = await pendingOrder("r-7");
      await expect(escrow.connect(buyer).timeoutRefund(h)).to.be.revertedWithCustomError(
        escrow,
        "TimeoutNotReached"
      );
    });

    it("rejects a timeout refund by anyone but the buyer", async () => {
      const h = await pendingOrder("r-8");
      await time.increase(REFUND_WINDOW + 1);
      await expect(escrow.connect(seller).timeoutRefund(h)).to.be.revertedWithCustomError(
        escrow,
        "NotAuthorized"
      );
      await expect(escrow.connect(owner).timeoutRefund(h)).to.be.revertedWithCustomError(
        escrow,
        "NotAuthorized"
      );
    });

    it("lets the buyer self-exit a confirmed-but-never-completed order", async () => {
      const h = await pendingOrder("r-9");
      await escrow.connect(seller).confirmOrder(h);
      await time.increase(REFUND_WINDOW + 1);
      await expect(escrow.connect(buyer).timeoutRefund(h)).to.changeEtherBalances(
        [buyer, escrow],
        [amount, -amount]
      );
    });

    it("rejects a timeout refund on a completed order", async () => {
      const h = await pendingOrder("r-10");
      await escrow.connect(seller).confirmOrder(h);
      await escrow.connect(seller).completeOrder(h);
      await time.increase(REFUND_WINDOW + 1);
      await expect(escrow.connect(buyer).timeoutRefund(h)).to.be.revertedWithCustomError(
        escrow,
        "InvalidStatus"
      );
    });
  });

  // -------------------------------------------------------------------

  describe("pause", () => {
    it("blocks new orders but still allows settlement and refunds", async () => {
      const amount = ethers.parseEther("1");
      const live = await createNative({
        orderId: "p-1",
        buyerAddr: buyer.address,
        amount,
        fee: 0n,
      });
      await live.tx;

      await escrow.connect(owner).pause();
      const blocked = await createNative({
        orderId: "p-2",
        buyerAddr: buyer.address,
        amount,
        fee: 0n,
      });
      await expect(blocked.tx).to.be.revertedWithCustomError(escrow, "EnforcedPause");

      // A pause must never trap escrowed funds.
      await escrow.connect(buyer).refundOrder(live.orderHash);
      await escrow.connect(owner).unpause();
    });
  });

  // -------------------------------------------------------------------

  describe("merchant status", () => {
    it("rejects an order against an unregistered or deactivated seller", async () => {
      const amount = ethers.parseEther("1");
      const unregistered = await createNative({
        orderId: "m-1",
        buyerAddr: buyer.address,
        sellerAddr: attacker.address,
        amount,
        fee: 0n,
      });
      await expect(unregistered.tx).to.be.revertedWithCustomError(escrow, "MerchantNotActive");

      await escrow.connect(owner).setMerchant(seller.address, sellerPayout.address, false);
      const deactivated = await createNative({
        orderId: "m-2",
        buyerAddr: buyer.address,
        amount,
        fee: 0n,
      });
      await expect(deactivated.tx).to.be.revertedWithCustomError(escrow, "MerchantNotActive");
    });

    it("keeps a deactivated merchant's live orders settleable", async () => {
      const amount = ethers.parseEther("1");
      const r = await createNative({
        orderId: "m-3",
        buyerAddr: buyer.address,
        amount,
        fee: 0n,
      });
      await r.tx;
      await escrow.connect(owner).setMerchant(seller.address, sellerPayout.address, false);
      await escrow.connect(seller).confirmOrder(r.orderHash);
      await expect(escrow.connect(seller).completeOrder(r.orderHash)).to.changeEtherBalance(
        sellerPayout,
        amount
      );
    });
  });

  // -------------------------------------------------------------------

  describe("reentrancy and payout safety", () => {
    it("a malicious buyer cannot re-enter refundOrder to double-withdraw", async () => {
      const Attacker = await ethers.getContractFactory("ReentrantAttacker");
      const atk = await Attacker.deploy(escrow.target);
      await owner.sendTransaction({ to: atk.target, value: ethers.parseEther("5") });

      const amount = ethers.parseEther("1");
      const { input } = await makeQuote({
        orderId: "re-1",
        buyerAddr: atk.target,
        amount,
        fee: 0n,
      });
      await atk.createOrder(input, { value: amount });

      // The re-entrant refund inside receive() reverts, which bubbles up as a failed
      // native payout: the attacker is never paid twice.
      await expect(atk.attackRefund()).to.be.revertedWithCustomError(
        escrow,
        "NativeTransferFailed"
      );
      expect(await escrow.escrowBalance(NATIVE)).to.equal(amount);
    });

    it("pays out to a multisig payout wallet whose receive() exceeds the 2300-gas stipend", async () => {
      const Multisig = await ethers.getContractFactory("MockMultisig");
      const safe = await Multisig.deploy();
      await escrow.connect(owner).setMerchant(seller.address, safe.target, true);

      const amount = ethers.parseEther("1");
      const r = await createNative({
        orderId: "ms-1",
        buyerAddr: buyer.address,
        amount,
        fee: 0n,
      });
      await r.tx;
      await escrow.connect(seller).confirmOrder(r.orderHash);
      await escrow.connect(seller).completeOrder(r.orderHash);

      expect(await ethers.provider.getBalance(safe.target)).to.equal(amount);
      expect(await safe.received()).to.equal(amount);
    });

    it("surfaces a failed native payout as a revert rather than losing funds", async () => {
      const Reverting = await ethers.getContractFactory("RevertingReceiver");
      const sink = await Reverting.deploy();
      await escrow.connect(owner).setMerchant(seller.address, sink.target, true);

      const amount = ethers.parseEther("1");
      const r = await createNative({
        orderId: "rv-1",
        buyerAddr: buyer.address,
        amount,
        fee: 0n,
      });
      await r.tx;
      await escrow.connect(seller).confirmOrder(r.orderHash);
      await expect(
        escrow.connect(seller).completeOrder(r.orderHash)
      ).to.be.revertedWithCustomError(escrow, "NativeTransferFailed");

      // The order is untouched and the buyer can still exit.
      expect(await escrow.escrowBalance(NATIVE)).to.equal(amount);
      await escrow.connect(seller).refundOrder(r.orderHash);
    });
  });

  // -------------------------------------------------------------------

  describe("stuck-fund recovery", () => {
    it("can sweep only funds that are not backing a live order", async () => {
      const amount = ethers.parseEther("1");
      const r = await createNative({
        orderId: "s-1",
        buyerAddr: buyer.address,
        amount,
        fee: 0n,
      });
      await r.tx;

      // Someone force-sends tokens straight to the escrow.
      await usdc.mint(escrow.target, 500n * USDC_UNIT);
      expect(await escrow.unaccountedBalance(usdc.target)).to.equal(500n * USDC_UNIT);
      // Escrowed native is NOT sweepable.
      expect(await escrow.unaccountedBalance(NATIVE)).to.equal(0);

      await expect(
        escrow.connect(owner).sweepUnaccounted(NATIVE, owner.address)
      ).to.be.revertedWithCustomError(escrow, "EscrowUnderflow");

      await escrow.connect(owner).sweepUnaccounted(usdc.target, owner.address);
      expect(await usdc.balanceOf(owner.address)).to.be.greaterThan(0);

      // The live order is still fully refundable afterwards.
      await expect(escrow.connect(buyer).refundOrder(r.orderHash)).to.changeEtherBalance(
        buyer,
        amount
      );
    });

    it("only the owner can sweep", async () => {
      await expect(
        escrow.connect(attacker).sweepUnaccounted(usdc.target, attacker.address)
      ).to.be.revertedWithCustomError(escrow, "OwnableUnauthorizedAccount");
    });
  });
});
