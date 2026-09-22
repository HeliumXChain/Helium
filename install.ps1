<#
.SYNOPSIS
  Helium one-click installer for Windows (mirrors install.sh).
.DESCRIPTION
  Installs the helium binary from a GitHub release (or a local .zip for
  private repos), wires PATH, opens mDNS + WireGuard firewall rules and can
  configure the borrower tunnel service.
.EXAMPLE
  powershell -ExecutionPolicy Bypass -File install.ps1 -ZipPath .\helium.zip
  powershell -ExecutionPolicy Bypass -File install.ps1 -Version v0.1.0-alpha13
  powershell -ExecutionPolicy Bypass -File install.ps1 -Borrower -Endpoint "192.168.1.84:51820" -PeerPubKey "BASE64=="
#>
param(
  [string]$Version = "v0.1.0-alpha13",
  [string]$ZipPath = "",
  [switch]$Borrower,
  [string]$Endpoint = "",
  [string]$PeerPubKey = "",
  [string]$InstallDir = "$env:LOCALAPPDATA\Helium\bin"
)

$ErrorActionPreference = "Stop"
$Repo = "HeliumXChain/Helium"

function Info($m)  { Write-Host "[INFO] $m" -ForegroundColor Cyan }
function Ok($m)    { Write-Host "[OK] $m" -ForegroundColor Green }
function Warn($m)  { Write-Host "[WARN] $m" -ForegroundColor Yellow }

function IsAdmin {
  ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
  ).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# 1. Binary ---------------------------------------------------------------
New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
$exe = Join-Path $InstallDir "helium.exe"
Get-Process helium -ErrorAction SilentlyContinue | ForEach-Object {
  Stop-Process -Id $_.Id -Force
  Warn "processus helium arrete (PID $($_.Id)) - relance-le apres install"
}
if ($ZipPath -ne "" -and (Test-Path $ZipPath)) {
  Info "install depuis archive locale: $ZipPath"
  $tmp = Join-Path $env:TEMP "helium-win-dl"
  if (Test-Path $tmp) { Remove-Item $tmp -Recurse -Force }
  Expand-Archive -Path $ZipPath -DestinationPath $tmp -Force
  $found = Get-ChildItem -Recurse -Filter "helium.exe" $tmp | Select-Object -First 1
  if (-not $found) { throw "helium.exe introuvable dans $ZipPath" }
  Copy-Item $found.FullName $exe -Force
} else {
  $url = "https://github.com/$Repo/releases/download/$Version/helium-$Version-windows-x86_64.zip"
  Info "telechargement: $url"
  $zip = Join-Path $env:TEMP "helium-win.zip"
  try {
    Invoke-WebRequest -Uri $url -OutFile $zip
  } catch {
    throw "telechargement impossible (repo prive ? connecte gh ou passe -ZipPath). $($_.Exception.Message)"
  }
  $tmp = Join-Path $env:TEMP "helium-win-dl"
  if (Test-Path $tmp) { Remove-Item $tmp -Recurse -Force }
  Expand-Archive -Path $zip -DestinationPath $tmp -Force
  $found = Get-ChildItem -Recurse -Filter "helium.exe" $tmp | Select-Object -First 1
  Copy-Item $found.FullName $exe -Force
}
Ok "binaire: $exe"
& $exe --version

# 2. PATH -----------------------------------------------------------------
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*Helium\bin*") {
  [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
  Ok "PATH mis a jour (rouvre le terminal)"
} else {
  Ok "PATH deja configure"
}

# 3. Firewall (mDNS multicast + WireGuard) ---------------------------------
if (-not (IsAdmin)) {
  Warn "pas admin : regles firewall ignorees (relance en admin pour mDNS/WireGuard)"
} else {
  $rules = @(
    @{ Name = "Helium-mDNS-In";  Direction = "Inbound";  Protocol = "UDP"; LocalPort = 5353 },
    @{ Name = "Helium-mDNS-Out"; Direction = "Outbound"; Protocol = "UDP"; RemotePort = 5353 },
    @{ Name = "Helium-WireGuard"; Direction = "Inbound"; Protocol = "UDP"; LocalPort = 51820 }
  )
  foreach ($r in $rules) {
    if (Get-NetFirewallRule -DisplayName $r.Name -ErrorAction SilentlyContinue) {
      Ok "firewall: $($r.Name) existe deja"
      continue
    }
    $params = @{
      DisplayName = $r.Name; Direction = $r.Direction; Program = $exe
      Protocol = $r.Protocol; Action = "Allow"
    }
    if ($r.ContainsKey("LocalPort"))  { $params["LocalPort"] = $r.LocalPort }
    if ($r.ContainsKey("RemotePort")) { $params["RemotePort"] = $r.RemotePort }
    New-NetFirewallRule @params | Out-Null
    Ok "firewall: $($r.Name) creee"
  }
}

# 4. Borrower tunnel (optionnel, admin requis) ------------------------------
if ($Borrower) {
  if ([string]::IsNullOrWhiteSpace($Endpoint) -or [string]::IsNullOrWhiteSpace($PeerPubKey)) {
    throw "-Borrower requiert -Endpoint IP:PORT et -PeerPubKey CLE"
  }
  if (-not (IsAdmin)) { throw "tunnel borrower = terminal admin requis" }
  $confDir = "C:\WireGuard"
  New-Item -ItemType Directory -Path $confDir -Force | Out-Null
  $priv = (wg genkey)
  $pub = ($priv | wg pubkey)
  @"
[Interface]
PrivateKey = $priv
Address = 10.0.0.2/24

[Peer]
PublicKey = $PeerPubKey
Endpoint = $Endpoint
AllowedIPs = 10.0.0.0/24
PersistentKeepalive = 25
"@ | Out-File "$confDir\helium-borrower.conf" -Encoding ascii
  wireguard /uninstalltunnelservice helium-borrower 2>$null
  wireguard /installtunnelservice "$confDir\helium-borrower.conf"
  sc.exe start 'WireGuardTunnel$helium-borrower'
  Ok "tunnel borrower actif (cle publique: $pub)"
  Write-Host "Sur provider: wg set helium-poc peer $pub allowed-ips 10.0.0.2/32"
}

Ok "Helium pret : helium status | helium dash"
