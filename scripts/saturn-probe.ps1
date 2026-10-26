param(
  [ValidateSet('prepare', 'release', 'patch', 'verify')][string]$Phase = 'prepare',
  [string]$PublicUrl = 'https://saturn.andka.id',
  [string]$ApiUrl = 'http://localhost:3000',
  [string]$Device = 'emulator-5554'
)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$probeRoot = Join-Path $projectRoot 'examples\saturn_probe'
$toolchainMarker = Join-Path $projectRoot '.tooling\shorebird-path.txt'
$toolchainRoot = if (Test-Path -LiteralPath $toolchainMarker) { (Get-Content -LiteralPath $toolchainMarker -Raw).Trim() } else { Join-Path $projectRoot '.tooling\shorebird' }
$shorebird = Join-Path $toolchainRoot 'bin\shorebird.bat'
$stateFile = Join-Path $projectRoot '.tooling\saturn-probe.json'
$keysDir = Join-Path $projectRoot '.tooling\saturn-signing'
$envFile = Join-Path $projectRoot 'server\.env'
if (!(Test-Path -LiteralPath $envFile)) { throw 'Create server/.env and start the server first; see server/README.md.' }
$tokenLine = Get-Content -LiteralPath $envFile | Where-Object { $_.StartsWith('ADMIN_TOKEN=') } | Select-Object -First 1
$env:SHOREBIRD_TOKEN = $tokenLine.Substring('ADMIN_TOKEN='.Length)
$env:SHOREBIRD_HOSTED_URL = $ApiUrl
$headers = @{ Authorization = 'Bearer ' + $env:SHOREBIRD_TOKEN }
function Api([string]$Method, [string]$Path, $Payload) {
  $requestArgs = @{ Method=$Method; Uri=($ApiUrl+$Path); Headers=$headers }
  if ($null -ne $Payload) { $requestArgs.ContentType='application/json'; $requestArgs.Body=($Payload | ConvertTo-Json -Depth 20 -Compress) }
  Invoke-RestMethod @requestArgs
}
function Invoke-Shorebird([string[]]$CommandArgs) {
  Push-Location $probeRoot
  try { & $shorebird @CommandArgs; if ($LASTEXITCODE -ne 0) { throw "Shorebird failed with exit code $LASTEXITCODE" } }
  finally { Pop-Location }
}
function Restart-Probe {
  & adb -s $Device reverse tcp:3001 tcp:3001 | Out-Null
  & adb -s $Device shell am force-stop id.andka.saturn_probe | Out-Null
  & adb -s $Device logcat -c
  & adb -s $Device shell am start -n id.andka.saturn_probe/.MainActivity | Out-Null
  if ($LASTEXITCODE -ne 0) { throw 'Cannot start probe on emulator' }
}
function Wait-Probe([string]$Pattern, [string]$EvidenceName) {
  $deadline = [DateTime]::UtcNow.AddSeconds(90)
  while ([DateTime]::UtcNow -lt $deadline) {
    $lines = & adb -s $Device logcat -d -s flutter:I '*:S'
    $needsSettledStatus = $EvidenceName -in @('baseline','patched','rolled-back')
    if (($lines -match $Pattern) -and (!$needsSettledStatus -or ($lines -match 'SATURN_PROBE status='))) {
      $evidenceDir = Join-Path $projectRoot 'artifacts\saturn'
      New-Item -ItemType Directory -Force -Path $evidenceDir | Out-Null
      $lines | Set-Content -LiteralPath (Join-Path $evidenceDir "$EvidenceName.log")
      & adb -s $Device shell screencap -p /sdcard/saturn-probe.png | Out-Null
      & adb -s $Device pull /sdcard/saturn-probe.png (Join-Path $evidenceDir "$EvidenceName.png") 2>$null | Out-Null
      $lines | Select-String 'SATURN_PROBE'
      return
    }
    Start-Sleep -Seconds 2
  }
  throw "Emulator evidence missing: $Pattern"
}
try {
  if ($Phase -eq 'prepare') {
    if (Test-Path -LiteralPath $stateFile) { $saved = Get-Content -LiteralPath $stateFile -Raw | ConvertFrom-Json }
    else {
      $app = Api POST '/api/v1/apps' @{ display_name='Saturn Android probe'; organization_id=1 }
      $saved = @{ app_id=$app.id }
      $saved | ConvertTo-Json | Set-Content -LiteralPath $stateFile
    }
    "app_id: $($saved.app_id)`nbase_url: $PublicUrl`nauto_update: false`n" | Set-Content -LiteralPath (Join-Path $probeRoot 'shorebird.yaml')
    New-Item -ItemType Directory -Force -Path $keysDir | Out-Null
    $openssl = 'C:\Program Files\Git\usr\bin\openssl.exe'
    if (!(Test-Path -LiteralPath (Join-Path $keysDir 'private.pem'))) {
      & $openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out (Join-Path $keysDir 'private.pem') 2>$null
      if ($LASTEXITCODE -ne 0) { throw 'Signing key generation failed' }
      & $openssl pkey -in (Join-Path $keysDir 'private.pem') -pubout -out (Join-Path $keysDir 'public.pem')
      if ($LASTEXITCODE -ne 0) { throw 'Public key extraction failed' }
    }
    Write-Output "Probe app prepared: $($saved.app_id). Signing keys remain in ignored .tooling/saturn-signing."
  } elseif ($Phase -eq 'release') {
    Invoke-Shorebird @('release','android','--artifact=apk','--target-platform=android-x64','--public-key-path', (Join-Path $keysDir 'public.pem'),'--','--dart-define=PROBE_MARKER=baseline')
    & adb -s $Device install -r (Join-Path $probeRoot 'build\app\outputs\flutter-apk\app-release.apk')
    if ($LASTEXITCODE -ne 0) { throw 'Probe installation failed' }
    Restart-Probe
    Wait-Probe 'SATURN_PROBE marker=baseline current=0.*available=true' 'baseline'
  } elseif ($Phase -eq 'patch') {
    Invoke-Shorebird @('patch','--platforms=android','--release-version=1.0.0+1','--track=staging','--private-key-path',(Join-Path $keysDir 'private.pem'),'--public-key-path',(Join-Path $keysDir 'public.pem'),'--','--target-platform=android-x64','--dart-define=PROBE_MARKER=patched')
  } else {
    $saved = Get-Content -LiteralPath $stateFile -Raw | ConvertFrom-Json
    $app = $saved.app_id
    $releases = (Api GET "/api/v1/apps/$app/releases" $null).releases
    $release = $releases | Where-Object version -eq '1.0.0+1' | Select-Object -First 1
    $patches = (Api GET "/api/v1/apps/$app/releases/$($release.id)/patches" $null).patches
    $patch = $patches | Sort-Object number -Descending | Select-Object -First 1
    if (!$patch) { throw 'Publish a patch before verification' }
    if ($patch.is_rolled_back) { Api POST "/api/v1/apps/$app/releases/$($release.id)/patches/$($patch.id)/rollforward" @{} | Out-Null }
    $stable = (Api GET "/api/v1/apps/$app/channels" $null) | Where-Object name -eq 'stable'
    Api POST "/api/v1/apps/$app/patches/promote" @{ patch_id=$patch.id; channel_id=$stable.id } | Out-Null
    Restart-Probe
    Wait-Probe "SATURN_PROBE downloaded_next=$($patch.number)" 'download'
    Restart-Probe
    Wait-Probe "SATURN_PROBE marker=patched current=$($patch.number).*available=true" 'patched'
    Api POST "/api/v1/apps/$app/releases/$($release.id)/patches/$($patch.id)/rollback" @{} | Out-Null
    Restart-Probe
    Wait-Probe 'SATURN_PROBE status=' 'rollback-check'
    Restart-Probe
    Wait-Probe 'SATURN_PROBE marker=baseline current=0.*available=true' 'rolled-back'
    Write-Output 'PASS: baseline -> signed patch download -> patched boot -> server rollback -> baseline boot.'
  }
} finally {
  Remove-Item Env:\SHOREBIRD_TOKEN -ErrorAction SilentlyContinue
  Remove-Item Env:\SHOREBIRD_HOSTED_URL -ErrorAction SilentlyContinue
}
