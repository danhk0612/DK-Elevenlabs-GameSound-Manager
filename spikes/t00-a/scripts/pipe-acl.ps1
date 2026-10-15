param([Parameter(Mandatory=$true)][string]$SourceSha,[Parameter(Mandatory=$true)][string]$OutFile)
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
 public static Evidence Check() {
  Evidence result=new Evidence();
  result.owner_sid=WindowsIdentity.GetCurrent().User.Value;
  string prefix=@"\\.\pipe\DK-T00-"+Guid.NewGuid();
  string own="D:P(A;;GA;;;"+result.owner_sid+")";
  IntPtr owner=Invalid,secure=Invalid,control=Invalid,token=IntPtr.Zero;
  try {
   // A built-in service token is a real, distinct local SID. No account,
   // password, profile, group membership or policy is created or changed.
   if(!LogonUserW("LocalService","NT AUTHORITY",IntPtr.Zero,5,0,out token))
    throw new Win32Exception(Marshal.GetLastWin32Error(),"LocalService logon failed");
   using(WindowsIdentity identity=new WindowsIdentity(token))result.client_sid=identity.User.Value;
   if(result.client_sid!="S-1-5-19" || result.client_sid==result.owner_sid)
    throw new Exception("Expected a distinct LocalService SID");
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
    throw new Exception("LocalService permissive control failed: "+result.permitted_control_open_error);
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
$result=[ordered]@{source_sha=$SourceSha;scope='standalone native named pipe candidate, not engine integration';remote_clients='PIPE_REJECT_REMOTE_CLIENTS flag set, remote machine not tested';different_real_user='LocalService SID token impersonation; ordinary second interactive account unverified';actual_key='not_used';account_changes='none'}
try {
 $result.native_evidence=[T00PipeAcl]::Check()
 $result.check='owner_open_and_localservice_denied_with_positive_control'
 $result.current_user_open='passed'
 $result.cross_identity_acl='passed: LocalService permitted control opened, owner-only pipe returned access denied'
 $result.result='passed_automated_only'
}
catch {$result.result='failed';$result.failure=$_.Exception.Message}
$result | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $OutFile -Encoding utf8
$result | ConvertTo-Json -Depth 4
if ($result.result -eq 'failed') {exit 1}
