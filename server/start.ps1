param([string]$PublicUrl='', [int]$Port=0)
$ErrorActionPreference='Stop'
$envPath=Join-Path $PSScriptRoot '.env'
if(!(Test-Path -LiteralPath $envPath)) {
  if(!$PublicUrl){$PublicUrl='https://saturn.andka.id'}
  if(!$Port){$Port=3000}
  $bytes=[Security.Cryptography.RandomNumberGenerator]::GetBytes(32)
  $token='sb_api_'+[Convert]::ToHexString($bytes).ToLowerInvariant()
  [IO.File]::WriteAllText($envPath,"ADMIN_TOKEN=$token`nPUBLIC_BASE_URL=$PublicUrl`nHOST_PORT=$Port`nRUST_LOG=info`n")
  Write-Output 'Created server/.env with a random management token. Open that file to unlock the dashboard.'
} else {
  $contents=Get-Content -LiteralPath $envPath -Raw
  if($PublicUrl){$contents=$contents -replace '(?m)^PUBLIC_BASE_URL=.*',("PUBLIC_BASE_URL="+$PublicUrl)}
  if($Port){
    if($contents -match '(?m)^HOST_PORT='){$contents=$contents -replace '(?m)^HOST_PORT=.*',("HOST_PORT="+$Port)}
    else {$contents+="`nHOST_PORT=$Port`n"}
  }
  [IO.File]::WriteAllText($envPath,$contents)
}
& docker compose --env-file $envPath -f (Join-Path $PSScriptRoot 'compose.yaml') up -d --build
if($LASTEXITCODE -ne 0){throw 'Container startup failed'}
$settings=Get-Content -LiteralPath $envPath
$publicSetting=($settings | Where-Object {$_.StartsWith('PUBLIC_BASE_URL=')} | Select-Object -First 1).Substring('PUBLIC_BASE_URL='.Length)
$portSetting=($settings | Where-Object {$_.StartsWith('HOST_PORT=')} | Select-Object -First 1)
if(!$portSetting){$portSetting='HOST_PORT=3000'}
Write-Output "Saturn is available at $publicSetting ($portSetting)."
