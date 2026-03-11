@echo off
echo ========================================
echo AeroLedger Quick Setup Script
echo ========================================
echo.

REM Check if PostgreSQL is installed
echo [1/6] Checking PostgreSQL...
psql --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: PostgreSQL not found. Please install from https://www.postgresql.org/download/
    pause
    exit /b 1
)
echo PostgreSQL found!
echo.

REM Check if Rust is installed
echo [2/6] Checking Rust...
cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Rust not found. Please install from https://rustup.rs/
    pause
    exit /b 1
)
echo Rust found!
echo.

REM Check if Node.js is installed
echo [3/6] Checking Node.js...
node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Node.js not found. Please install from https://nodejs.org/
    pause
    exit /b 1
)
echo Node.js found!
echo.

REM Create .env file if it doesn't exist
echo [4/6] Setting up environment...
if not exist .env (
    echo Creating .env file...
    (
        echo DATABASE_URL=postgresql://postgres:postgres@localhost/aeroledger
        echo JWT_SECRET=change-this-secret-key-in-production
        echo SMTP_HOST=smtp.gmail.com
        echo SMTP_PORT=587
        echo SMTP_USERNAME=your-email@gmail.com
        echo SMTP_PASSWORD=your-app-password
        echo SMTP_FROM=noreply@aeroledger.com
        echo BTC_WALLET_ADDRESS=bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh
        echo ETH_WALLET_ADDRESS=0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
        echo SOL_WALLET_ADDRESS=7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
        echo USDT_WALLET_ADDRESS=0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
        echo USDC_WALLET_ADDRESS=0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
        echo BNB_WALLET_ADDRESS=0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
        echo ETHERSCAN_API_KEY=your_key_here
        echo BSCSCAN_API_KEY=your_key_here
        echo SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
    ) > .env
    echo .env file created! Please edit it with your actual values.
    echo.
) else (
    echo .env file already exists.
    echo.
)

REM Install SQLx CLI
echo [5/6] Installing SQLx CLI...
cargo install sqlx-cli --no-default-features --features postgres
echo.

REM Create database
echo [6/6] Setting up database...
echo Please enter your PostgreSQL password when prompted.
psql -U postgres -c "CREATE DATABASE aeroledger;" 2>nul
if %errorlevel% equ 0 (
    echo Database created successfully!
) else (
    echo Database might already exist or check your PostgreSQL password.
)
echo.

REM Run migrations
echo Running database migrations...
sqlx migrate run
echo.

echo ========================================
echo Setup Complete!
echo ========================================
echo.
echo Next steps:
echo 1. Edit .env file with your actual values
echo 2. Run: cargo run (in one terminal)
echo 3. Run: cd frontend-luxury ^&^& npm install ^&^& npm run dev (in another terminal)
echo 4. Open: http://localhost:3000
echo.
echo For smart contracts setup, see COMPLETE_SETUP_GUIDE.md
echo.
pause
