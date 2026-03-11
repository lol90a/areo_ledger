const hre = require("hardhat");

async function main() {
  console.log("Deploying AeroLedgerEscrow...");

  const AeroLedgerEscrow = await hre.ethers.getContractFactory("AeroLedgerEscrow");
  const escrow = await AeroLedgerEscrow.deploy();

  await escrow.waitForDeployment();
  const address = await escrow.getAddress();

  console.log(`AeroLedgerEscrow deployed to: ${address}`);

  const network = hre.network.name;
  let usdtAddress, usdcAddress;

  if (network === "mainnet") {
    usdtAddress = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
    usdcAddress = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
  } else if (network === "bsc") {
    usdtAddress = "0x55d398326f99059fF775485246999027B3197955";
    usdcAddress = "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d";
  } else if (network === "sepolia") {
    usdtAddress = "0x7169D38820dfd117C3FA1f22a697dBA58d90BA06";
    usdcAddress = "0x94a9D9AC8a22534E3FaCa9F4e7F2E2cf85d5E4C8";
  } else if (network === "bscTestnet") {
    usdtAddress = "0x337610d27c682E347C9cD60BD4b3b107C9d34dDd";
    usdcAddress = "0x64544969ed7EBf5f083679233325356EbE738930";
  }

  if (usdtAddress) {
    const tx1 = await escrow.addSupportedToken(usdtAddress);
    await tx1.wait();
    console.log(`Added USDT: ${usdtAddress}`);
  }

  if (usdcAddress) {
    const tx2 = await escrow.addSupportedToken(usdcAddress);
    await tx2.wait();
    console.log(`Added USDC: ${usdcAddress}`);
  }

  console.log(`\nContract: ${address}`);
  console.log(`Verify: npx hardhat verify --network ${network} ${address}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
