param([Parameter(Mandatory=$true)][string]$BinDirectory,[Parameter(Mandatory=$true)][ValidateSet('status','shutdown')][string]$Operation)
$ErrorActionPreference='Stop'
# Pass JSON through the Windows process command line explicitly. Windows PowerShell 5.1
# removes embedded quotes when calling native EXEs directly with a JSON string.
$start=New-Object System.Diagnostics.ProcessStartInfo
$start.FileName=Join-Path (Resolve-Path $BinDirectory).Path 'spike-cli.exe'
$start.Arguments='{\"op\":\"'+$Operation+'\"}'
$start.UseShellExecute=$false
$start.RedirectStandardOutput=$true
$start.RedirectStandardError=$true
$process=[Diagnostics.Process]::Start($start)
$output=$process.StandardOutput.ReadToEnd()
$diagnostic=$process.StandardError.ReadToEnd()
$process.WaitForExit()
if ($process.ExitCode -ne 0 -or $diagnostic) {throw 'Fixture CLI failed; inspect the JSON command result.'}
$output.TrimEnd()
