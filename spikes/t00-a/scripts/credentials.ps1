# Dummy credentials only. Never accept a real API Key or print a secret/blob.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Security
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class T00Cred {
 [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)] public struct C {
  public uint Flags, Type; public string TargetName, Comment; public System.Runtime.InteropServices.ComTypes.FILETIME LastWritten;
  public uint CredentialBlobSize; public IntPtr CredentialBlob; public uint Persist, AttributeCount; public IntPtr Attributes;
  public string TargetAlias, UserName;
 }
 [DllImport("advapi32.dll", EntryPoint="CredWriteW", CharSet=CharSet.Unicode, SetLastError=true)] public static extern bool Write(ref C c, uint f);
 [DllImport("advapi32.dll", EntryPoint="CredReadW", CharSet=CharSet.Unicode, SetLastError=true)] public static extern bool Read(string t,uint y,uint f,out IntPtr p);
 [DllImport("advapi32.dll", EntryPoint="CredDeleteW", CharSet=CharSet.Unicode, SetLastError=true)] public static extern bool Delete(string t,uint y,uint f);
 [DllImport("advapi32.dll")] public static extern void CredFree(IntPtr p);
}
'@
$dummy = [Text.Encoding]::UTF8.GetBytes(('T00-DUMMY-' + [guid]::NewGuid()))
$target = 'DK-T00-Spike-' + [guid]::NewGuid()
$blob = [Runtime.InteropServices.Marshal]::AllocHGlobal($dummy.Length)
$read = [IntPtr]::Zero
$written = $false
try {
 [Runtime.InteropServices.Marshal]::Copy($dummy,0,$blob,$dummy.Length)
 $c = New-Object T00Cred+C
 $c.Type=1; $c.TargetName=$target; $c.CredentialBlobSize=$dummy.Length; $c.CredentialBlob=$blob; $c.Persist=2; $c.UserName='fixture'
 if (-not [T00Cred]::Write([ref]$c,0)) { throw 'CredWrite failed' }
 $written=$true
 if (-not [T00Cred]::Read($target,1,0,[ref]$read)) { throw 'CredRead failed' }
 $r=[Runtime.InteropServices.Marshal]::PtrToStructure($read,[type][T00Cred+C])
 $got=New-Object byte[] $r.CredentialBlobSize
 [Runtime.InteropServices.Marshal]::Copy($r.CredentialBlob,$got,0,$got.Length)
 if ([Convert]::ToBase64String($got) -cne [Convert]::ToBase64String($dummy)) { throw 'Credential mismatch' }
 $scope=[Security.Cryptography.DataProtectionScope]::CurrentUser
 $encrypted=[Security.Cryptography.ProtectedData]::Protect($dummy,$null,$scope)
 $decrypted=[Security.Cryptography.ProtectedData]::Unprotect($encrypted,$null,$scope)
 if ([Convert]::ToBase64String($decrypted) -cne [Convert]::ToBase64String($dummy)) { throw 'DPAPI mismatch' }
} finally {
 if ($read -ne [IntPtr]::Zero) { [T00Cred]::CredFree($read) }
 [Runtime.InteropServices.Marshal]::FreeHGlobal($blob)
 if ($written -and -not [T00Cred]::Delete($target,1,0)) { throw 'Dummy cleanup failed' }
}
@{credential_manager='roundtrip_deleted'; dpapi_current_user='roundtrip_memory_only'; cross_user='unverified'; actual_key='not_used'} | ConvertTo-Json -Compress
