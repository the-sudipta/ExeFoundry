param(
  [string]$InputBat,      # e.g. .\scripts\MyTool.bat
  [string]$OutputExe,     # e.g. .\MyTool.exe
  [string]$Icon,          # e.g. .\scripts\icon\bat_to_exe.ico
  [switch]$WinExe         # hide console for the produced EXE
)

function Normalize([string]$p) {
  if ([string]::IsNullOrWhiteSpace($p)) { return $null }
  $p = $p.Trim().Trim('"')
  try { return (Resolve-Path -LiteralPath $p -ErrorAction Stop).Path } catch { return $p }
}

# ---------- Interactive fallback ----------
if ([string]::IsNullOrWhiteSpace($InputBat)) {
  $InputBat  = Read-Host 'Input BAT path (e.g. .\scripts\Auto Reload Until Website is Up.bat)'
}
if ([string]::IsNullOrWhiteSpace($OutputExe)) {
  $OutputExe = Read-Host 'Output EXE path (e.g. .\EdgeBeacon.exe)'
}
if ([string]::IsNullOrWhiteSpace($Icon)) {
  $Icon = Read-Host 'Icon ICO path (optional, Enter to skip)'
  if ([string]::IsNullOrWhiteSpace($Icon)) { $Icon = $null }
}

$InputBat  = Normalize $InputBat
$OutputExe = Normalize $OutputExe
$Icon      = Normalize $Icon

if (-not (Test-Path -LiteralPath $InputBat)) {
  Write-Error "Input BAT not found: $InputBat"; exit 1
}

# Ensure .exe extension + output folder exists
if ([IO.Path]::GetExtension($OutputExe) -ne ".exe") {
  $OutputExe = $OutputExe + ".exe"
}
[IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($OutputExe))) | Out-Null

# ---------- Embed BAT into C# stub ----------
$bytes = [IO.File]::ReadAllBytes($InputBat)
$b64   = [Convert]::ToBase64String($bytes)

$src = @"
using System;
using System.IO;
using System.Diagnostics;
class Program {
  static int Main(string[] args) {
    try {
      string b64 = @"$b64";
      byte[] data = Convert.FromBase64String(b64);
      string tmp = Path.Combine(Path.GetTempPath(), "bat2exe_" + Guid.NewGuid().ToString("N"));
      Directory.CreateDirectory(tmp);
      string bat = Path.Combine(tmp, "payload.bat");
      File.WriteAllBytes(bat, data);

      string argLine = "";
      if (args != null && args.Length > 0) {
        foreach (var a in args) {
          if (a == null) continue;
          string e = a.Replace("\"", "\\\"");
          argLine += " \"" + e + "\"";
        }
      }

      var psi = new ProcessStartInfo("cmd.exe", "/c \"" + bat + "\"" + argLine) {
        UseShellExecute = false,
        RedirectStandardOutput = false,
        RedirectStandardError = false,
        WorkingDirectory = tmp,
        CreateNoWindow = false
      };
      var p = Process.Start(psi);
      p.WaitForExit();
      int code = p.ExitCode;
      try { Directory.Delete(tmp, true); } catch {}
      return code;
    } catch (Exception ex) {
      Console.Error.WriteLine(ex.ToString());
      return 1;
    }
  }
}
"@

$srcPath = Join-Path $env:TEMP ("batstub_" + [guid]::NewGuid().ToString("N") + ".cs")
[IO.File]::WriteAllText($srcPath, $src)

# ---------- Find csc.exe ----------
function Find-Csc {
  $cands = @(
    "$env:WINDIR\Microsoft.NET\Framework64\v4.0.30319\csc.exe",
    "$env:WINDIR\Microsoft.NET\Framework\v4.0.30319\csc.exe"
  )
  foreach ($p in $cands) { if (Test-Path $p) { return $p } }
  $roots = @("$env:WINDIR\Microsoft.NET\Framework64","$env:WINDIR\Microsoft.NET\Framework")
  foreach ($r in $roots) {
    if (-not (Test-Path $r)) { continue }
    Get-ChildItem $r -Directory -Filter "v4*" | Sort-Object Name -Descending |
      ForEach-Object {
        $p = Join-Path $_.FullName "csc.exe"
        if (Test-Path $p) { return $p }
      }
  }
  return $null
}
$csc = Find-Csc
if (-not $csc) { Write-Error "Could not locate csc.exe (.NET Framework v4.x)"; exit 2 }

# ---------- Compile ----------
$target = "exe"; if ($WinExe.IsPresent) { $target = "winexe" }
$outArg = "/out:`"$OutputExe`""
$args = @("/nologo","/optimize+","/target:$target",$outArg,$srcPath)
if ($Icon) { $args += "/win32icon:`"$Icon`"" }

& $csc @args
$exit = $LASTEXITCODE
Remove-Item $srcPath -ErrorAction SilentlyContinue

if ($exit -ne 0) { Write-Error "Compilation failed (exit $exit)"; exit $exit }
Write-Host "OK: Built $OutputExe"
