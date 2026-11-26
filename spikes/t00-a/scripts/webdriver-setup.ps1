param([string]$OutDirectory='evidence')
$ErrorActionPreference='Stop'
New-Item -ItemType Directory -Force $OutDirectory | Out-Null
$runtime = Get-ChildItem "${env:ProgramFiles(x86)}\Microsoft\EdgeWebView\Application\*\msedgewebview2.exe" | Sort-Object { [version]$_.VersionInfo.ProductVersion } -Descending | Select-Object -First 1
if (-not $runtime) {throw 'WebView2 runtime not found'}
$version = $runtime.VersionInfo.ProductVersion
$zip = Join-Path $env:RUNNER_TEMP 't00-edgedriver.zip'
$url="https://msedgedriver.microsoft.com/$version/edgedriver_win64.zip"
try {Invoke-WebRequest $url -OutFile $zip}
catch {
 $major=$version.Split('.')[0]
 $match=(Invoke-WebRequest "https://msedgedriver.microsoft.com/LATEST_RELEASE_${major}_WINDOWS").Content.Trim()
 if (($match.Split('.')[0..2] -join '.') -ne ($version.Split('.')[0..2] -join '.')) {throw 'Driver/runtime build mismatch'}
 $url="https://msedgedriver.microsoft.com/$match/edgedriver_win64.zip"
 Invoke-WebRequest $url -OutFile $zip
}
$dest=Join-Path $env:RUNNER_TEMP 't00-edgedriver'
Expand-Archive $zip -DestinationPath $dest -Force
$dest | Out-File $env:GITHUB_PATH -Append -Encoding utf8
@{webview2=$version;driver_url=$url;zip_sha256=(Get-FileHash $zip -Algorithm SHA256).Hash;driver_version=(& (Join-Path $dest 'msedgedriver.exe') --version)} | ConvertTo-Json | Set-Content (Join-Path $OutDirectory 'webdriver-tools.json') -Encoding utf8
