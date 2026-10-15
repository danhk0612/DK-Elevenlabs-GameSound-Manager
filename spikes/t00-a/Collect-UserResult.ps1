$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
$sha = (Get-Content source-sha.txt -Raw).Trim()
$env:T00_SPIKE_ROOT = Join-Path $env:TEMP ('T00 사용자 실기 ' + [guid]::NewGuid())
python scripts/verify.py --bin . --sha $sha --out user-automatic.json
if ($LASTEXITCODE -ne 0) { throw '자동 검사 실패: user-automatic.json을 확인하세요.' }
New-Item -ItemType Directory -Force $env:T00_SPIKE_ROOT | Out-Null
Copy-Item user-automatic.wav (Join-Path $env:T00_SPIKE_ROOT '합성 음원.wav') -ErrorAction SilentlyContinue
$engine = Start-Process .\spike-engine.exe -PassThru
try {
  Start-Process .\spike-gui.exe -Wait
  & .\spike-cli.exe '{"op":"status"}'
  Start-Process .\spike-gui.exe -Wait
  $result = [ordered]@{source_sha=$sha; os=[Environment]::OSVersion.VersionString; executor='사용자 이름/별칭 기입'; gui_visible='미검증'; gui_common_command='미검증'; close_job_reconnect='미검증'; synthetic_audio_audible='미검증'; notes='화면·청감 관찰을 작성해 PR에 첨부'}
  $result | ConvertTo-Json | Set-Content user-observation.json -Encoding UTF8
} finally {
  & .\spike-cli.exe '{"op":"shutdown"}'
  $engine.WaitForExit(5000) | Out-Null
}
Write-Host 'user-automatic.json 및 user-observation.json을 검토 후 PR에 첨부하세요.'
