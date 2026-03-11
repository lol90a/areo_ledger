@echo off
echo ========================================
echo Starting AeroLedger Platform
echo ========================================
echo.

REM Check if .env exists
if not exist .env (
    echo ERROR: .env file not found!
    echo Please run setup.bat first or create .env manually.
    pause
    exit /b 1
)

REM Check if database is accessible
echo Checking database connection...
psql -U postgres -d aeroledger -c "SELECT 1;" >nul 2>&1
if %errorlevel% neq 0 (
    echo WARNING: Cannot connect to database. Make sure PostgreSQL is running.
    echo.
)

echo Starting backend and frontend...
echo.
echo Backend will run on: http://127.0.0.1:8080
echo Frontend will run on: http://localhost:3000
echo.
echo Press Ctrl+C to stop all services.
echo.

REM Start backend in new window
start "AeroLedger Backend" cmd /k "cargo run"

REM Wait a bit for backend to start
timeout /t 5 /nobreak >nul

REM Start frontend in new window
start "AeroLedger Frontend" cmd /k "cd frontend-luxury && npm run dev"

echo.
echo Both services are starting in separate windows.
echo Close those windows or press Ctrl+C to stop them.
echo.
pause
