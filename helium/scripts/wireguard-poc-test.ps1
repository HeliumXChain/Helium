# WireGuard POC Test Script
# Tests P2P tunnel establishment between two Helium nodes
# Run this on two different machines or VMs

param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("provider", "borrower")]
    [string]$Role,
    
    [string]$PeerEndpoint = "",
    [string]$PeerPublicKey = ""
)

$ErrorActionPreference = "Stop"

# Colors for output
function Write-Info($msg) { Write-Host "[INFO] $msg" -ForegroundColor Cyan }
function Write-Success($msg) { Write-Host "[OK] $msg" -ForegroundColor Green }
function Write-Error($msg) { Write-Host "[ERR] $msg" -ForegroundColor Red }
function Write-Step($msg) { Write-Host "`n>> $msg" -ForegroundColor Yellow }

# Check if WireGuard is installed
Write-Step "Checking WireGuard installation..."
try {
    $wgVersion = wg --version 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Success "WireGuard found: $wgVersion"
    } else {
        throw "WireGuard not found"
    }
} catch {
    Write-Error "WireGuard not installed. Install from: https://download.wireguard.com/windows-client/"
    exit 1
}

# Generate keypair
Write-Step "Generating WireGuard keypair..."
$privateKey = wg genkey
$publicKey = $privateKey | wg pubkey
Write-Info "Public Key: $publicKey"
Write-Info "Private Key: $($privateKey.Substring(0,8))..."

# Configuration based on role
$interfaceName = "helium-poc"
$listenPort = 51820

if ($Role -eq "provider") {
    Write-Step "Configuring as PROVIDER (listening on port $listenPort)..."
    
    $config = @"
[Interface]
PrivateKey = $privateKey
Address = 10.0.0.1/24
ListenPort = $listenPort

# Will add peer config after borrower connects
"@
    
    Write-Info "Provider config generated"
    Write-Info "Your PUBLIC KEY: $publicKey"
    Write-Info "Share this with the borrower"
    
} else {
    if (-not $PeerEndpoint -or -not $PeerPublicKey) {
        Write-Error "Borrower role requires -PeerEndpoint and -PeerPublicKey parameters"
        Write-Info "Example: .\wireguard-poc-test.ps1 -Role borrower -PeerEndpoint '192.168.1.100:51820' -PeerPublicKey 'abcd1234...'"
        exit 1
    }
    
    Write-Step "Configuring as BORROWER (connecting to $PeerEndpoint)..."
    
    $config = @"
[Interface]
PrivateKey = $privateKey
Address = 10.0.0.2/24

[Peer]
PublicKey = $PeerPublicKey
Endpoint = $PeerEndpoint
AllowedIPs = 10.0.0.0/24
PersistentKeepalive = 25
"@
    
    Write-Info "Borrower config generated"
}

# Save config
$configPath = "$env:TEMP\helium-poc.conf"
$config | Out-File -FilePath $configPath -Encoding UTF8
Write-Info "Config saved to: $configPath"

# Display config
Write-Host "`n=== WireGuard Configuration ===" -ForegroundColor Magenta
Write-Host $config
Write-Host "=================================`n" -ForegroundColor Magenta

# Import and test
Write-Step "Importing tunnel configuration..."
try {
    # Remove existing interface if present
    wg-quick down $interfaceName 2>$null
    
    # Import config
    wg-quick up $configPath
    
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Tunnel interface '$interfaceName' created successfully!"
        
        # Show interface status
        Write-Step "Interface Status:"
        wg show $interfaceName
        
        # Test connectivity
        if ($Role -eq "borrower") {
            Write-Step "Testing connectivity to provider (10.0.0.1)..."
            Start-Sleep -Seconds 2
            
            $pingResult = Test-Connection -ComputerName "10.0.0.1" -Count 3 -ErrorAction SilentlyContinue
            if ($pingResult) {
                Write-Success "Ping successful! Tunnel is working."
                Write-Info "Latency: $($pingResult | Select-Object -ExpandProperty ResponseTime | Measure-Object -Average | Select-Object -ExpandProperty Average) ms"
            } else {
                Write-Error "Ping failed. Check firewall rules and NAT configuration."
                Write-Info "Try: ping 10.0.0.1"
                Write-Info "Check: wg show"
            }
        }
        
        Write-Host "`n=== SUCCESS ===" -ForegroundColor Green
        Write-Host "WireGuard tunnel established!" -ForegroundColor Green
        Write-Host "Interface: $interfaceName" -ForegroundColor Cyan
        Write-Host "Your IP: $(if ($Role -eq 'provider') { '10.0.0.1' } else { '10.0.0.2' })" -ForegroundColor Cyan
        
        if ($Role -eq "provider") {
            Write-Host "`nNext steps for borrower:" -ForegroundColor Yellow
            Write-Host "1. Share your public key: $publicKey" -ForegroundColor White
            Write-Host "2. Share your public IP: $(Invoke-RestMethod -Uri 'https://api.ipify.org')" -ForegroundColor White
            Write-Host "3. Wait for borrower to connect" -ForegroundColor White
        }
        
    } else {
        Write-Error "Failed to create tunnel interface"
        exit 1
    }
    
} catch {
    Write-Error "Error creating tunnel: $_"
    Write-Info "You may need to run as Administrator"
    exit 1
}

# Keep tunnel up and wait
Write-Host "`n" + ("=" * 50) -ForegroundColor Yellow
Write-Host "TUNNEL IS ACTIVE - Press Ctrl+C to stop" -ForegroundColor Yellow
Write-Host ("=" * 50) + "`n" -ForegroundColor Yellow

# Monitor handshake
try {
    while ($true) {
        $status = wg show $interfaceName latest-handshakes 2>$null
        if ($status) {
            Write-Host "$(Get-Date -Format 'HH:mm:ss') - $status" -ForegroundColor Gray
        }
        Start-Sleep -Seconds 5
    }
} finally {
    Write-Host "`nShutting down tunnel..." -ForegroundColor Yellow
    wg-quick down $interfaceName 2>$null
    Write-Success "Tunnel closed"
}
