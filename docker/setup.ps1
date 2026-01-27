# =============================================================================
# Amail Docker Setup Script (PowerShell)
# =============================================================================
# Usage: .\setup.ps1 [start|stop|restart|logs|status]
# =============================================================================

param(
    [Parameter(Position=0)]
    [ValidateSet("start", "stop", "restart", "logs", "status", "init")]
    [string]$Action = "start"
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

function Write-Header {
    param([string]$Message)
    Write-Host "`n=== $Message ===" -ForegroundColor Cyan
}

function Test-DockerRunning {
    try {
        docker info | Out-Null
        return $true
    } catch {
        Write-Host "Docker is not running. Please start Docker Desktop." -ForegroundColor Red
        return $false
    }
}

function Start-Stack {
    Write-Header "Starting Amail Docker Stack"
    
    if (-not (Test-DockerRunning)) { return }
    
    Set-Location $ScriptDir
    docker-compose up -d
    
    Write-Host "`nServices started:" -ForegroundColor Green
    Write-Host "  - Stalwart Mail:  http://localhost:4040 (admin UI)"
    Write-Host "  - PostgreSQL:     localhost:5432"
    Write-Host "  - Redis:          localhost:6379"
    Write-Host "`nSMTP Ports:"
    Write-Host "  - 25   (SMTP)"
    Write-Host "  - 587  (Submission/STARTTLS)"
    Write-Host "  - 465  (SMTPS)"
    Write-Host "`nIMAP Ports:"
    Write-Host "  - 143  (IMAP)"
    Write-Host "  - 993  (IMAPS)"
}

function Stop-Stack {
    Write-Header "Stopping Amail Docker Stack"
    
    if (-not (Test-DockerRunning)) { return }
    
    Set-Location $ScriptDir
    docker-compose down
    
    Write-Host "All services stopped." -ForegroundColor Yellow
}

function Restart-Stack {
    Stop-Stack
    Start-Stack
}

function Show-Logs {
    Write-Header "Docker Logs (Ctrl+C to exit)"
    
    if (-not (Test-DockerRunning)) { return }
    
    Set-Location $ScriptDir
    docker-compose logs -f
}

function Show-Status {
    Write-Header "Amail Docker Stack Status"
    
    if (-not (Test-DockerRunning)) { return }
    
    Set-Location $ScriptDir
    docker-compose ps
}

function Initialize-Mailboxes {
    Write-Header "Initializing Mailboxes"
    
    # Read from .env if exists
    $envFile = Join-Path (Split-Path $ScriptDir -Parent) ".env"
    $domain = "localhost"
    $mailboxes = @("inbox", "support", "hr", "legal", "finance", "executive")
    
    if (Test-Path $envFile) {
        Get-Content $envFile | ForEach-Object {
            if ($_ -match "^MAIL_DOMAIN=(.+)$") { $domain = $Matches[1] }
            if ($_ -match "^MAIL_BOXES=(.+)$") { $mailboxes = $Matches[1] -split "," }
        }
    }
    
    Write-Host "Domain: $domain"
    Write-Host "Mailboxes: $($mailboxes -join ', ')"
    
    # Create mailboxes via Stalwart API
    $baseUrl = "http://localhost:4040/api"
    $creds = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("admin:admin123"))
    
    foreach ($box in $mailboxes) {
        $boxName = $box.Trim()
        $email = "$boxName@$domain"
        
        Write-Host "Creating mailbox: $email" -ForegroundColor Gray
        
        try {
            $body = @{
                name = $boxName
                email = $email
                password = "amail_$boxName"
            } | ConvertTo-Json
            
            Invoke-RestMethod -Uri "$baseUrl/account" -Method Post `
                -Headers @{ Authorization = "Basic $creds"; "Content-Type" = "application/json" } `
                -Body $body -ErrorAction SilentlyContinue
                
            Write-Host "  Created: $email" -ForegroundColor Green
        } catch {
            Write-Host "  Skipped (may already exist): $email" -ForegroundColor Yellow
        }
    }
    
    Write-Host "`nMailbox initialization complete." -ForegroundColor Green
}

# Main
switch ($Action) {
    "start"   { Start-Stack }
    "stop"    { Stop-Stack }
    "restart" { Restart-Stack }
    "logs"    { Show-Logs }
    "status"  { Show-Status }
    "init"    { Initialize-Mailboxes }
}
