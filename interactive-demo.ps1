# Interactive ShapeRunner Demo
$env:PATH = ".\protoc\bin;$env:PATH"

Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "   ShapeRunner Interactive Demo                            " -ForegroundColor White
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Server should be running in background window." -ForegroundColor Yellow
Write-Host ""
Write-Host "Available commands:" -ForegroundColor Cyan
Write-Host "  1. Run with example input" -ForegroundColor Gray
Write-Host "  2. Run with custom JSON input" -ForegroundColor Gray
Write-Host "  3. Exit" -ForegroundColor Gray
Write-Host ""

while ($true) {
    $choice = Read-Host "Select option (1-3)"
    
    if ($choice -eq "1") {
        Write-Host ""
        Write-Host "Running with example input..." -ForegroundColor Yellow
        Write-Host "INPUT:" -ForegroundColor Cyan
        Get-Content examples/feature-design-input.json | Write-Host -ForegroundColor Gray
        Write-Host ""
        Write-Host "OUTPUT:" -ForegroundColor Cyan
        cargo run --release --bin shape-runner-cli -- --input examples/feature-design-input.json 2>&1 | Where-Object { $_ -notmatch "^    (Finished|Compiling|warning)" }
        Write-Host ""
    }
    elseif ($choice -eq "2") {
        Write-Host ""
        Write-Host "Enter JSON input (press Enter twice when done):" -ForegroundColor Yellow
        $lines = @()
        $emptyCount = 0
        while ($true) {
            $line = Read-Host
            if ([string]::IsNullOrWhiteSpace($line)) {
                $emptyCount++
                if ($emptyCount -ge 2) { break }
            } else {
                $emptyCount = 0
                $lines += $line
            }
        }
        $jsonInput = $lines -join "`n"
        
        $tempFile = "temp-input-$(Get-Date -Format 'yyyyMMddHHmmss').json"
        $jsonInput | Out-File -FilePath $tempFile -Encoding utf8
        
        Write-Host ""
        Write-Host "Running with custom input..." -ForegroundColor Yellow
        Write-Host "INPUT:" -ForegroundColor Cyan
        $jsonInput | Write-Host -ForegroundColor Gray
        Write-Host ""
        Write-Host "OUTPUT:" -ForegroundColor Cyan
        cargo run --release --bin shape-runner-cli -- --input $tempFile 2>&1 | Where-Object { $_ -notmatch "^    (Finished|Compiling|warning)" }
        Remove-Item $tempFile -ErrorAction SilentlyContinue
        Write-Host ""
    }
    elseif ($choice -eq "3") {
        Write-Host ""
        Write-Host "Exiting demo. Server is still running in background." -ForegroundColor Yellow
        break
    }
    else {
        Write-Host "Invalid option. Please select 1-3." -ForegroundColor Red
    }
}
