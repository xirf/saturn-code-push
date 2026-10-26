param([string]$ToolingRoot=(Join-Path $env:LOCALAPPDATA 'SaturnCodePush'))
$ErrorActionPreference='Stop'
$projectRoot=Split-Path -Parent $PSScriptRoot
$revision='2bb03a04ad157f014f16fe588f97317de7ad9b02'
$sbRoot=Join-Path $ToolingRoot 'shorebird'
New-Item -ItemType Directory -Force -Path $ToolingRoot,(Join-Path $projectRoot '.tooling') | Out-Null
if(!(Test-Path -LiteralPath (Join-Path $sbRoot '.git'))) {
  if(Test-Path -LiteralPath $sbRoot){throw 'Toolchain directory exists without Git metadata; choose a new ToolingRoot.'}
  git init $sbRoot
  if($LASTEXITCODE -ne 0){throw 'Git initialization failed'}
  git -C $sbRoot remote add origin https://github.com/shorebirdtech/shorebird.git
  git -C $sbRoot config core.longpaths true
  git -C $sbRoot fetch --depth 1 origin $revision
  if($LASTEXITCODE -ne 0){throw 'Toolchain fetch failed'}
  git -C $sbRoot checkout --detach FETCH_HEAD
  if($LASTEXITCODE -ne 0){throw 'Toolchain checkout failed'}
}
$actual=(git -C $sbRoot rev-parse HEAD).Trim()
if($actual -ne $revision){throw "Expected pinned Shorebird revision $revision; found $actual. Existing checkout was preserved."}
Set-Content -LiteralPath (Join-Path $projectRoot '.tooling\shorebird-path.txt') -Value $sbRoot
$reaRoot=Join-Path $ToolingRoot 'rea'
if(!(Test-Path -LiteralPath (Join-Path $reaRoot 'node_modules\.bin\rea.cmd'))) {
  npm.cmd install --prefix $reaRoot rea-agents@6.7.0
  if($LASTEXITCODE -ne 0){throw 'REA CLI installation failed'}
}
Set-Content -LiteralPath (Join-Path $projectRoot '.tooling\rea-path.txt') -Value $reaRoot
# Apply long-path support only to this process and its Git children.
$oldGitCount=$env:GIT_CONFIG_COUNT
$oldGitKey=$env:GIT_CONFIG_KEY_0
$oldGitValue=$env:GIT_CONFIG_VALUE_0
try {
  if($oldGitCount){throw 'GIT_CONFIG_COUNT is already configured; preserve it and bootstrap Shorebird manually.'}
  $env:GIT_CONFIG_COUNT='1'; $env:GIT_CONFIG_KEY_0='core.longpaths'; $env:GIT_CONFIG_VALUE_0='true'
  & (Join-Path $sbRoot 'bin\shorebird.bat') --version
  if($LASTEXITCODE -ne 0){throw 'Shorebird bootstrap failed'}
} finally {
  $env:GIT_CONFIG_COUNT=$oldGitCount
  $env:GIT_CONFIG_KEY_0=$oldGitKey
  $env:GIT_CONFIG_VALUE_0=$oldGitValue
}
Write-Output "Pinned toolchain ready in $ToolingRoot. The REA skill is installed separately into your agent's skills directory."
