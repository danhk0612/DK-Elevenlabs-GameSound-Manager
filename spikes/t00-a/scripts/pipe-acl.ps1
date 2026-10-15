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
 [DllImport("kernel32.dll")] static extern IntPtr GetCurrentThread();
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool ImpersonateAnonymousToken(IntPtr t);
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
  IntPtr h=CreateFileW(p,0xC0000000,0,IntPtr.Zero,3,0,IntPtr.Zero);
  if(h==Invalid)return Marshal.GetLastWin32Error();CloseHandle(h);return 0;
 }
 public static string Check() {
  string sid=WindowsIdentity.GetCurrent().User.Value;
  string prefix=@"\\.\pipe\DK-T00-"+Guid.NewGuid();string own="D:P(A;;GA;;;"+sid+")";
  IntPtr owner=Invalid,secure=Invalid,control=Invalid;
  try {
   owner=Server(prefix+"-owner",own);
   if(Open(prefix+"-owner")!=0)throw new Exception("Current user pipe open failed");
   secure=Server(prefix+"-secure",own);control=Server(prefix+"-control",own+"(A;;GA;;;AN)");
   int denied,allowed;
   if(!ImpersonateAnonymousToken(GetCurrentThread()))throw new Win32Exception();
   try {denied=Open(prefix+"-secure");allowed=Open(prefix+"-control");}
   finally {if(!RevertToSelf())throw new Win32Exception();}
   if(allowed==5 && denied==5)return "inconclusive_anonymous_control_access_denied";
   if(allowed!=0)throw new Exception("Anonymous permissive control failed: "+allowed);
   if(denied!=5)throw new Exception("Owner-only pipe did not return access denied: "+denied);
   return "owner_open_and_anonymous_denied_with_positive_control";
  } finally {
   if(owner!=Invalid)CloseHandle(owner);if(secure!=Invalid)CloseHandle(secure);if(control!=Invalid)CloseHandle(control);
  }
 }
}
'@
$result=[ordered]@{source_sha=$SourceSha;scope='standalone native named pipe candidate, not engine integration';remote_clients='PIPE_REJECT_REMOTE_CLIENTS flag set, remote machine not tested';different_real_user='unverified';actual_key='not_used'}
try {
 $result.check=[T00PipeAcl]::Check()
 if ($result.check -eq 'inconclusive_anonymous_control_access_denied') {
  $result.result='partial_environment_blocked'
  $result.current_user_open='passed'
  $result.cross_identity_acl='unverified: anonymous permitted control also denied access'
 } else {$result.result='passed_automated_only'}
}
catch {$result.result='failed';$result.failure=$_.Exception.Message}
$result | ConvertTo-Json | Set-Content -LiteralPath $OutFile -Encoding utf8
$result | ConvertTo-Json
if ($result.result -eq 'failed') {exit 1}
