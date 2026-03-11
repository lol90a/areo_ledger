@echo off
echo ========================================
echo Installing Ethereum Contract Dependencies
echo ========================================
echo.

cd /d "%~dp0"

echo Checking Node.js...
node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Node.js not found!
    echo Please install from: https://nodejs.org/
    pause
    exit /b 1
)
echo Node.js found!
echo.

echo Installing npm packages...
echo This may take a few minutes...
echo.

npm install

if %errorlevel% equ 0 (
    echo.
    echo ========================================
    echo Installation Complete!
    echo ========================================
    echo.
    echo Next steps:
    echo 1. Create .env file with your keys
    echo 2. Run: npm run compile
    echo 3. Run: npm run deploy:sepolia
    echo.
) else (
    echo.
    echo ========================================
    echo Installation Failed!
    echo ========================================
    echo.
    echo Try these solutions:
    echo 1. Delete node_modules and package-lock.json
    echo 2. Run: npm cache clean --force
    echo 3. Run this script again
    echo.
)

pause
