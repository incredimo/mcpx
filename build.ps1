# automatically build all servers and copy them to the release folder
# Create release folder if it doesn't exist, clean it if it does
if (!(Test-Path -Path .\release)) {
    New-Item -ItemType Directory -Force -Path .\release
} else {
    Get-ChildItem -Path .\release -File | ForEach-Object {
        try {
            Stop-Process -Name $_.BaseName -ErrorAction SilentlyContinue
            Remove-Item $_.FullName -Force
        } catch {
            Write-Warning "Could not remove file: $($_.FullName). Error: $($_.Exception.Message)"
        }
    }
}

# Build and copy all servers
Get-ChildItem -Directory | Where-Object { $_.Name -ne "release" } | ForEach-Object {
    $folderName = $_.Name
    Set-Location -Path ".\$folderName"
    
    # Build release version
    cargo build --release
    
    # Copy the binary to the release folder
    $binaryName = if ($folderName -eq "jupyter") { "mpcx-$folderName" } else { "mcpx-$folderName" }
    $sourcePath = ".\target\release\$binaryName.exe"
    if (Test-Path $sourcePath) {
        try {
            $destPath = "..\release\$binaryName.exe"
            Stop-Process -Name $binaryName -ErrorAction SilentlyContinue
            Copy-Item -Path $sourcePath -Destination $destPath -Force
        } catch {
            Write-Warning "Could not copy file: $sourcePath. Error: $($_.Exception.Message)"
        }
    } else {
        Write-Warning "Binary not found: $sourcePath"
    }
    
    # Return to the parent directory
    Set-Location ..
}

# Display build results
Get-ChildItem -Path .\release
