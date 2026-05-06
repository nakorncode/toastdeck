using System.Runtime.InteropServices;

namespace ToastDeckA;

internal static class NativeMethods
{
    [DllImport("user32.dll", EntryPoint = "GetWindowLongW")]
    internal static extern int GetWindowLong(IntPtr hWnd, int nIndex);

    [DllImport("user32.dll", EntryPoint = "SetWindowLongW")]
    internal static extern int SetWindowLong(IntPtr hWnd, int nIndex, int dwNewLong);
}
