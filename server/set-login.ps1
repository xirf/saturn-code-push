param([string]$Username='admin')
$ErrorActionPreference='Stop'
$envPath=Join-Path $PSScriptRoot '.env'
$compose=Join-Path $PSScriptRoot 'compose.yaml'
if(!(Test-Path -LiteralPath $envPath)){throw 'Start Saturn first to generate server/.env.'}
if(!$Username -or $Username.Length -gt 128 -or $Username -match '[\r\n]'){throw 'Use a username of 1 to 128 characters.'}
$securePassword=Read-Host 'New password (at least 12 characters)' -AsSecureString
$confirmPassword=Read-Host 'Confirm password' -AsSecureString
$plainPassword=[Net.NetworkCredential]::new('',$securePassword).Password
$confirm=[Net.NetworkCredential]::new('',$confirmPassword).Password
try {
  if($plainPassword -ne $confirm){throw 'Passwords do not match.'}
  if($plainPassword.Length -lt 12 -or $plainPassword.Length -gt 1024){throw 'Use a password of 12 to 1024 characters.'}
  $hash=($plainPassword | & docker compose --env-file $envPath -f $compose run --rm --no-deps -T code-push saturn-code-push --hash-password | Out-String).Trim()
  if($LASTEXITCODE -ne 0 -or !$hash.StartsWith('$argon2id$')){throw 'Password hashing failed. Build the current Docker image first.'}
  $contents=Get-Content -LiteralPath $envPath -Raw
  $contents=[regex]::Replace($contents,'(?m)^ADMIN_(USERNAME|PASSWORD_HASH)=.*\r?\n?','')
  $contents+="`nADMIN_USERNAME=$Username`nADMIN_PASSWORD_HASH='"+$hash+"'`n"
  [IO.File]::WriteAllText($envPath,$contents)
  & docker compose --env-file $envPath -f $compose up -d
  if($LASTEXITCODE -ne 0){throw 'Could not restart Saturn with the new credentials.'}
  $handoff=Join-Path (Split-Path -Parent $PSScriptRoot) '.tooling\dashboard-login.txt'
  if(Test-Path -LiteralPath $handoff){Remove-Item -LiteralPath $handoff}
  Write-Output "Updated dashboard login for $Username. Existing browser sessions are invalidated. CLI API keys are unchanged."
} finally {
  $plainPassword=$null; $confirm=$null; $securePassword.Dispose(); $confirmPassword.Dispose()
}
