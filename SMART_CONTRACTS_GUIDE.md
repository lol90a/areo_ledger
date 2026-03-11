# Smart Contracts Deployment Guide

## Overview
AeroLedger now includes smart contracts for trustless escrow payments on Ethereum, BSC, and Solana.

## Contracts Created

### 1. Ethereum/BSC Escrow Contract
**Location**: `contracts/ethereum/AeroLedgerEscrow.sol`

**Features**:
- Escrow for ETH/BNB and ERC20 tokens (USDT, USDC)
- Order creation with automatic locking
- Admin confirmation and completion
- 24-hour refund window
- Reentrancy protection
- Event emission for tracking

**Functions**:
- `createOrder(orderId, token)` - Create order with native token
- `createOrderERC20(orderId, token, amount)` - Create order with ERC20
- `confirmOrder(orderHash)` - Admin confirms order
- `completeOrder(orderHash)` - Admin completes and releases funds
- `refundOrder(orderHash)` - Refund within 24 hours
- `cancelOrder(orderHash)` - Admin cancels order

### 2. Solana Escrow Program
**Location**: `contracts/solana/programs/aeroledger-escrow/src/lib.rs`

**Features**:
- SPL token escrow
- PDA-based order accounts
- Status tracking (Pending, Confirmed, Completed, Refunded)
- 24-hour refund window
- Event emission

**Instructions**:
- `create_order` - Lock tokens in escrow
- `confirm_order` - Admin confirms
- `complete_order` - Release to admin
- `refund_order` - Return to buyer

## Deployment Instructions

### Ethereum/BSC Deployment

#### 1. Install Dependencies
```bash
cd contracts/ethereum
npm install
```

#### 2. Configure Environment
Create `.env` file:
```env
PRIVATE_KEY=your_private_key_here
ETH_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR-API-KEY
SEPOLIA_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR-API-KEY
BSC_RPC_URL=https://bsc-dataseed1.binance.org
BSC_TESTNET_RPC_URL=https://data-seed-prebsc-1-s1.binance.org:8545
ETHERSCAN_API_KEY=your_etherscan_api_key
BSCSCAN_API_KEY=your_bscscan_api_key
```

#### 3. Compile Contracts
```bash
npm run compile
```

#### 4. Deploy to Testnet
```bash
# Ethereum Sepolia
npm run deploy:sepolia

# BSC Testnet
npm run deploy:bscTestnet
```

#### 5. Deploy to Mainnet
```bash
# Ethereum Mainnet
npm run deploy:mainnet

# BSC Mainnet
npm run deploy:bsc
```

#### 6. Verify Contract
```bash
npx hardhat verify --network sepolia <CONTRACT_ADDRESS>
```

### Solana Deployment

#### 1. Install Anchor
```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest
```

#### 2. Build Program
```bash
cd contracts/solana
anchor build
```

#### 3. Get Program ID
```bash
solana address -k target/deploy/aeroledger_escrow-keypair.json
```

#### 4. Update Program ID
Replace `AeroLedgerEscrowProgramID111111111111111111` in:
- `programs/aeroledger-escrow/src/lib.rs`
- `Anchor.toml`

#### 5. Rebuild
```bash
anchor build
```

#### 6. Deploy to Devnet
```bash
solana config set --url devnet
anchor deploy
```

#### 7. Deploy to Mainnet
```bash
solana config set --url mainnet-beta
anchor deploy
```

## Integration with Backend

### Update `.env.production`
```env
# Ethereum/BSC Contract
ETH_ESCROW_CONTRACT=0x...
BSC_ESCROW_CONTRACT=0x...

# Solana Program
SOLANA_ESCROW_PROGRAM=...
```

### Backend Integration Points

#### 1. Create Order with Smart Contract
When user creates order, call smart contract:

**Ethereum/BSC**:
```rust
// Use ethers-rs or web3
let contract = EscrowContract::new(address, provider);
let tx = contract.create_order(order_id, token_address).send().await?;
```

**Solana**:
```rust
// Use anchor-client
let program = client.program(program_id);
let tx = program
    .request()
    .accounts(CreateOrder { ... })
    .args(instruction::CreateOrder { order_id, amount })
    .send()?;
```

#### 2. Confirm Order
After payment verification:
```rust
contract.confirm_order(order_hash).send().await?;
```

#### 3. Complete Order
After delivery:
```rust
contract.complete_order(order_hash).send().await?;
```

#### 4. Refund Order
If cancelled:
```rust
contract.refund_order(order_hash).send().await?;
```

## Testing

### Ethereum/BSC Tests
```bash
cd contracts/ethereum
npx hardhat test
```

### Solana Tests
```bash
cd contracts/solana
anchor test
```

## Security Considerations

1. **Multi-sig Wallet**: Use Gnosis Safe for admin operations
2. **Timelock**: Add timelock for critical functions
3. **Audit**: Get contracts audited before mainnet
4. **Rate Limiting**: Implement rate limits on contract calls
5. **Monitoring**: Set up alerts for contract events

## Gas Optimization

### Ethereum/BSC
- Batch operations when possible
- Use EIP-1559 for gas estimation
- Monitor gas prices before transactions

### Solana
- Optimize account sizes
- Use compute budget instructions
- Batch transactions when possible

## Contract Addresses (After Deployment)

### Testnet
- Ethereum Sepolia: `TBD`
- BSC Testnet: `TBD`
- Solana Devnet: `TBD`

### Mainnet
- Ethereum: `TBD`
- BSC: `TBD`
- Solana: `TBD`

## Support

For contract issues:
1. Check transaction on block explorer
2. Review contract events
3. Verify gas/compute limits
4. Contact development team

---

**IMPORTANT**: Test thoroughly on testnet before mainnet deployment. Never deploy with real funds without audit.
