# ShapeRunner Demo Script
# This script starts the servers and runs the demo

Write-Host "=== ShapeRunner Demo ===" -ForegroundColor Cyan
Write-Host ""

# Set up protoc path
$env:PATH = ".\protoc\bin;$env:PATH"

# Step 1: Start Mock LLM Server
Write-Host "[1/3] Starting Mock LLM Server..." -ForegroundColor Yellow
$mockProcess = Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd '$PWD'; `$env:PATH='$PWD\protoc\bin;' + `$env:PATH; Write-Host '=== Mock LLM Server ===' -ForegroundColor Cyan; Write-Host 'Listening on http://localhost:8080/llm' -ForegroundColor Green; cargo run --release --bin mock-llm-server" -PassThru -WindowStyle Normal
Start-Sleep -Seconds 5

# Step 2: Start ShapeRunner Server  
Write-Host "[2/3] Starting ShapeRunner Server..." -ForegroundColor Yellow
$env:LLM_BASE_URL = "http://localhost:8080/llm"
$serverProcess = Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd '$PWD'; `$env:PATH='$PWD\protoc\bin;' + `$env:PATH; `$env:LLM_BASE_URL='http://localhost:8080/llm'; Write-Host '=== ShapeRunner Server ===' -ForegroundColor Cyan; Write-Host 'Listening on http://0.0.0.0:50051' -ForegroundColor Green; Write-Host 'LLM: http://localhost:8080/llm' -ForegroundColor Green; cargo run --release --bin shape-runner" -PassThru -WindowStyle Normal
Start-Sleep -Seconds 8

# Step 3: Run CLI Client
Write-Host "[3/3] Running CLI Client..." -ForegroundColor Yellow
Write-Host ""
Write-Host "Input:" -ForegroundColor Cyan
Get-Content examples/feature-design-input.json
Write-Host ""
Write-Host "Output:" -ForegroundColor Cyan
cargo run --release --bin shape-runner-cli -- --input examples/feature-design-input.json

Write-Host ""
Write-Host "Demo complete! Servers are still running in the background windows." -ForegroundColor Green
Write-Host "Press any key to stop servers and exit..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

# Cleanup
Write-Host "Stopping servers..." -ForegroundColor Yellow
Stop-Process -Id $mockProcess.Id -Force -ErrorAction SilentlyContinue
Stop-Process -Id $serverProcess.Id -Force -ErrorAction SilentlyContinue
Write-Host "Done!" -ForegroundColor Green

