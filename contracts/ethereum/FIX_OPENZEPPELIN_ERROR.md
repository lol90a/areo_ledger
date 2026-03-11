# Fix for OpenZeppelin Import Error

## The Problem
The error occurs because OpenZeppelin contracts haven't been installed yet.

## Solution

### Step 1: Navigate to contracts directory
```bash
cd contracts\ethereum
```

### Step 2: Install dependencies
```bash
npm install
```

This will install:
- @openzeppelin/contracts
- hardhat
- @nomicfoundation/hardhat-toolbox
- dotenv

### Step 3: Verify installation
```bash
# Check if node_modules exists
dir node_modules\@openzeppelin\contracts
```

### Step 4: Compile contracts
```bash
npm run compile
```

## If Still Getting Errors

### Option 1: Install OpenZeppelin separately
```bash
npm install @openzeppelin/contracts@5.0.0
```

### Option 2: Clear cache and reinstall
```bash
rmdir /s /q node_modules
del package-lock.json
npm install
```

### Option 3: Use specific Hardhat version
```bash
npm install hardhat@2.19.0 --save-dev
npm install @nomicfoundation/hardhat-toolbox@4.0.0 --save-dev
npm install @openzeppelin/contracts@5.0.0
```

## Expected Output After npm install

You should see:
```
added 500+ packages
```

And these folders should exist:
- `node_modules/@openzeppelin/contracts/`
- `node_modules/hardhat/`
- `node_modules/@nomicfoundation/`

## Test Compilation

After installing, run:
```bash
npm run compile
```

You should see:
```
Compiled 1 Solidity file successfully
```

## Common Issues

### Issue: npm not found
**Solution**: Install Node.js from https://nodejs.org/

### Issue: Permission denied
**Solution**: Run as administrator or use:
```bash
npm install --no-optional
```

### Issue: Network timeout
**Solution**: Use different registry:
```bash
npm install --registry=https://registry.npmjs.org/
```

## Quick Fix Script

Create `install.bat` in `contracts/ethereum/`:
```batch
@echo off
echo Installing Ethereum contract dependencies...
npm install
if %errorlevel% equ 0 (
    echo Success! Now run: npm run compile
) else (
    echo Error installing dependencies
)
pause
```

Then just run:
```bash
install.bat
```
