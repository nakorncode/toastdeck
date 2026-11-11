# Tight feedback loop for the "Dismiss all does not clear the launch card" bug.
#
# Launches ToastDesk, waits for the launch card, then posts a real WM_LBUTTONUP at the centre of the
# "Dismiss all" button into the overlay window. The overlay hides itself when its card list becomes
# empty, so IsWindowVisible is an observable pass/fail signal:
#   green (fixed)  -> overlay hidden after the click
#   red   (buggy)  -> overlay still visible, launch card still there

param(
    [string]$Exe = "$env:LOCALAPPDATA\ToastDesk.v3\ToastDeskV3.exe",
    [int]$WaitForCard = 20
)

$ErrorActionPreference = 'Stop'

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class Ov {
  [DllImport("user32.dll", CharSet=CharSet.Unicode)]
  public static extern IntPtr FindWindowW(string cls, string title);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
}
'@

# Layout constants, mirrored from native/src/overlay.rs so the click lands on the button.
$OVERLAY_WIDTH = 380
$PAD = 12
$BUTTON_HEIGHT = 26
$WM_LBUTTONUP = 0x0202

function Get-Overlay {
    # The overlay window is created with class ToastDeskOverlay and title ToastDesk.
    [Ov]::FindWindowW('ToastDeskOverlay', 'ToastDesk')
}

function Send-Click([int]$x, [int]$y) {
    # WM_LBUTTONUP lparam packs client coordinates: low word x, high word y.
    $lp = [IntPtr](($y -shl 16) -bor ($x -band 0xFFFF))
    [void][Ov]::PostMessageW((Get-Overlay), $WM_LBUTTONUP, [IntPtr]::Zero, $lp)
}

Write-Output "== stopping any running copy so the launch card is guaranteed to appear =="
Get-Process ToastDeskV3 -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 900

Write-Output "== launching $Exe =="
$proc = Start-Process $Exe -PassThru

$hwnd = [IntPtr]::Zero
for ($i = 0; $i -lt ($WaitForCard * 2); $i++) {
    Start-Sleep -Milliseconds 500
    $hwnd = Get-Overlay
    if ($hwnd -ne [IntPtr]::Zero -and [Ov]::IsWindowVisible($hwnd)) { break }
}

if ($hwnd -eq [IntPtr]::Zero) { throw "overlay window never appeared (is the launch card disabled?)" }
if (-not [Ov]::IsWindowVisible($hwnd)) { throw "overlay window exists but is not visible" }

$rect = New-Object Ov+RECT
[void][Ov]::GetWindowRect($hwnd, [ref]$rect)
"   overlay hwnd=$hwnd visible=$([Ov]::IsWindowVisible($hwnd)) rect=($($rect.Left),$($rect.Top))-($($rect.Right),$($rect.Bottom))"

# The button is at the top-right of the overlay: left = WIDTH - PAD - buttonWidth, top = PAD.
# buttonWidth depends on the measured text; clicking its vertical centre and a point a little left of
# the right edge hits it for any reasonable width. The row spans a wide band, so aim at mid-height and
# the middle of the row.
$clickX = $OVERLAY_WIDTH - $PAD - 60
$clickY = $PAD + [int]($BUTTON_HEIGHT / 2)
"   clicking Dismiss all at client ($clickX,$clickY)"

Send-Click -x $clickX -y $clickY
Start-Sleep -Milliseconds 1200

$visibleAfter = [Ov]::IsWindowVisible((Get-Overlay))
"   overlay visible after click: $visibleAfter"

Get-Process ToastDeskV3 -ErrorAction SilentlyContinue | Stop-Process -Force

if (-not $visibleAfter) {
    Write-Output "RESULT: PASS - Dismiss all cleared the launch card and the overlay hid itself"
    exit 0
}
Write-Output "RESULT: FAIL - the launch card survived Dismiss all (the reported bug)"
exit 1
