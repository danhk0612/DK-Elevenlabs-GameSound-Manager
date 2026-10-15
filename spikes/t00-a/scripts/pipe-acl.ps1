param([Parameter(Mandatory=$true)][string]$SourceSha,[Parameter(Mandatory=$true)][string]$OutFile,[switch]$UseEphemeralCiUser)
$ErrorActionPreference='Stop'
Add-Type @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Security.Principal;
public static class T00PipeAcl {
 [StructLayout(LayoutKind.Sequential)] struct SA { public int nLength; public IntPtr lpSecurityDescriptor; public int bInheritHandle; }
 [DllImport("advapi32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool ConvertStringSecurityDescriptorToSecurityDescriptorW(string s,uint r,out IntPtr p,out uint n);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr CreateNamedPipeW(string n,uint o,uint m,uint i,uint a,uint b,uint t,ref SA s);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr CreateFileW(string n,uint a,uint s,IntPtr p,uint c,uint f,IntPtr t);
 [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
 [DllImport("kernel32.dll")] static extern IntPtr LocalFree(IntPtr h);
 [DllImport("advapi32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool LogonUserW(string user,string domain,IntPtr password,uint type,uint provider,out IntPtr token);
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool ImpersonateLoggedOnUser(IntPtr token);
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool RevertToSelf();
 static readonly IntPtr Invalid=new IntPtr(-1);
 static IntPtr Server(string path,string sddl) {
  IntPtr d;uint n;
  if(!ConvertStringSecurityDescriptorToSecurityDescriptorW(sddl,1,out d,out n))throw new Win32Exception();
  try {
   SA a=new SA();a.nLength=Marshal.SizeOf(typeof(SA));a.lpSecurityDescriptor=d;
   IntPtr h=CreateNamedPipeW(path,0x80003,8,1,1024,1024,0,ref a);
   if(h==Invalid)throw new Win32Exception();return h;
  } finally {LocalFree(d);}
 }
 static int Open(string p) {
  IntPtr h=CreateFileW(p,0x00120083,0,IntPtr.Zero,3,0,IntPtr.Zero);
  if(h==Invalid)return Marshal.GetLastWin32Error();CloseHandle(h);return 0;
 }
 public sealed class Evidence {
  public string owner_sid;
  public string client_sid;
  public string impersonated_sid;
  public string reverted_sid;
  public int owner_open_error;
  public int protected_open_error;
  public int permitted_control_open_error;
 }
 public static Evidence Check(string user,string domain,IntPtr password,uint logonType,string expectedSid) {
  Evidence result=new Evidence();
  result.owner_sid=WindowsIdentity.GetCurrent().User.Value;
  string prefix=@"\\.\pipe\DK-T00-"+Guid.NewGuid();
  string own="D:P(A;;GA;;;"+result.owner_sid+")";
  IntPtr owner=Invalid,secure=Invalid,control=Invalid,token=IntPtr.Zero;
  try {
   // Use a real distinct SID, never NEW_CREDENTIALS (same local identity).
   if(!LogonUserW(user,domain,password,logonType,0,out token))
    throw new Win32Exception(Marshal.GetLastWin32Error());
   using(WindowsIdentity identity=new WindowsIdentity(token))result.client_sid=identity.User.Value;
   if(result.client_sid!=expectedSid || result.client_sid==result.owner_sid)
    throw new Exception("Expected the distinct test user SID");
   owner=Server(prefix+"-owner",own);
   result.owner_open_error=Open(prefix+"-owner");
   if(result.owner_open_error!=0)throw new Exception("Current user pipe open failed: "+result.owner_open_error);
   secure=Server(prefix+"-secure",own);
   // Same flags/rights as protected pipe; only the explicit client ACE differs.
   // Client rights exclude FILE_CREATE_PIPE_INSTANCE (same bit as APPEND_DATA).
   control=Server(prefix+"-control",own+"(A;;0x00120083;;;"+result.client_sid+")");
   if(!ImpersonateLoggedOnUser(token))throw new Win32Exception();
   try {
    result.impersonated_sid=WindowsIdentity.GetCurrent().User.Value;
    if(result.impersonated_sid!=result.client_sid)throw new Exception("Client impersonation identity mismatch");
    result.protected_open_error=Open(prefix+"-secure");
    result.permitted_control_open_error=Open(prefix+"-control");
   } finally {if(!RevertToSelf())throw new Win32Exception();}
   result.reverted_sid=WindowsIdentity.GetCurrent().User.Value;
   if(result.reverted_sid!=result.owner_sid)throw new Exception("Owner identity not restored");
   if(result.permitted_control_open_error!=0)
    throw new Exception("Distinct user permissive control failed: "+result.permitted_control_open_error);
   if(result.protected_open_error!=5)
    throw new Exception("Owner-only pipe did not return access denied: "+result.protected_open_error);
   return result;
  } finally {
   if(owner!=Invalid)CloseHandle(owner);if(secure!=Invalid)CloseHandle(secure);if(control!=Invalid)CloseHandle(control);
   if(token!=IntPtr.Zero)CloseHandle(token);
  }
 }
}
'@
$result=[ordered]@{source_sha=$SourceSha;scope='standalone native named pipe candidate, not engine integration';remote_clients='PIPE_REJECT_REMOTE_CLIENTS flag set, remote machine not tested';actual_key='not_used'}
$testUser=$null
$password=$null
$passwordPointer=[IntPtr]::Zero
try {
 if ($UseEphemeralCiUser) {
  if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted' -or $env:RUNNER_OS -ne 'Windows') {
   throw 'Ephemeral account mode requires an explicitly selected GitHub-hosted Windows CI runner.'
  }
  $name='t00acl'+[guid]::NewGuid().ToString('N').Substring(0,10)
  $password=[Security.SecureString]::new()
  # The random fixture password never enters command arguments, logs or reports.
  $random=[byte[]]::new(32)
  $rng=[Security.Cryptography.RandomNumberGenerator]::Create()
  try {$rng.GetBytes($random)} finally {$rng.Dispose()}
  foreach ($b in $random) {$password.AppendChar([char](65+($b % 26)))}
  foreach ($c in 'a1!'.ToCharArray()) {$password.AppendChar($c)}
  [Array]::Clear($random,0,$random.Length)
  $password.MakeReadOnly()
  $testUser=New-LocalUser -Name $name -Password $password -Description 'T00-A temporary ACL fixture' -AccountNeverExpires
  Add-LocalGroupMember -SID 'S-1-5-32-545' -Member $testUser.Name
  $passwordPointer=[Runtime.InteropServices.Marshal]::SecureStringToGlobalAllocUnicode($password)
  $result.different_real_user='temporary standard local account, interactive logon token impersonation'
  $result.account_changes='temporary account only on GitHub-hosted runner; cleanup required'
  $result.native_evidence=[T00PipeAcl]::Check($testUser.Name,'.',$passwordPointer,2,$testUser.SID.Value)
 } else {
  $result.different_real_user='LocalService token candidate; no account changes'
  $result.account_changes='none'
  $result.native_evidence=[T00PipeAcl]::Check('LocalService','NT AUTHORITY',[IntPtr]::Zero,5,'S-1-5-19')
 }
 $result.check='owner_open_and_distinct_user_denied_with_positive_control'
 $result.current_user_open='passed'
 $result.cross_identity_acl='passed: distinct SID permitted control opened, owner-only pipe returned access denied'
 $result.result='passed_automated_only'
}
catch {
 $failure=$_.Exception.GetBaseException()
 $result.result='failed';$result.failure=$failure.Message
 if ($failure -is [ComponentModel.Win32Exception]) {$result.native_error=$failure.NativeErrorCode}
}
finally {
 if ($passwordPointer -ne [IntPtr]::Zero) {[Runtime.InteropServices.Marshal]::ZeroFreeGlobalAllocUnicode($passwordPointer)}
 if ($null -ne $password) {$password.Dispose()}
 if ($null -ne $testUser) {
  try {
   Remove-LocalUser -SID $testUser.SID
   if (Get-LocalUser -SID $testUser.SID -ErrorAction SilentlyContinue) {throw 'Fixture account still exists'}
   $result.account_cleanup='passed: temporary account removed'
  } catch {$result.result='failed';$result.cleanup_failure=$_.Exception.Message}
 }
}
$result | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $OutFile -Encoding utf8
$result | ConvertTo-Json -Depth 4
if ($result.result -eq 'failed') {exit 1}
