using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using Windows.Data.Xml.Dom;
using Windows.UI.Notifications;

namespace ToastDeckA;

public sealed class WindowsNotificationPlatform
{
    public const string AppId = "NakornCode.ToastDeckA";

    private readonly string shortcutPath;

    public WindowsNotificationPlatform()
    {
        var programsPath = Environment.GetFolderPath(Environment.SpecialFolder.StartMenu);
        shortcutPath = Path.Combine(programsPath, "Programs", "ToastDeck-A.lnk");
    }

    public WindowsNotificationPlatformResult Initialize()
    {
        try
        {
            NativeMethods.SetCurrentProcessExplicitAppUserModelID(AppId);
            EnsureShortcut();

            return new WindowsNotificationPlatformResult(
                true,
                $"Windows notification source registered as {AppId}.");
        }
        catch (Exception ex)
        {
            WriteDiagnosticLog(ex);

            return new WindowsNotificationPlatformResult(
                false,
                $"Windows notification source registration failed: {ex.Message}");
        }
    }

    private static void WriteDiagnosticLog(Exception exception)
    {
        var logDir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "ToastDeck-A");
        Directory.CreateDirectory(logDir);

        File.WriteAllText(
            Path.Combine(logDir, "notification-platform.log"),
            exception.ToString());
    }

    public WindowsNotificationPlatformResult SendTestNotification()
    {
        try
        {
            var xml = new XmlDocument();
            xml.LoadXml($"""
                <toast launch="action=test">
                  <visual>
                    <binding template="ToastGeneric">
                      <text>ToastDeck-A test</text>
                      <text>Sent to Windows Notification Center at {DateTimeOffset.Now:HH:mm:ss}.</text>
                    </binding>
                  </visual>
                  <actions>
                    <action content="Open ToastDeck-A" arguments="action=open" activationType="foreground" />
                  </actions>
                </toast>
                """);

            var toast = new ToastNotification(xml);
            ToastNotificationManager.CreateToastNotifier(AppId).Show(toast);

            return new WindowsNotificationPlatformResult(
                true,
                "Sent a real Windows toast. If capture is enabled, it should appear in the list shortly.");
        }
        catch (Exception ex)
        {
            return new WindowsNotificationPlatformResult(false, $"Failed to send Windows toast: {ex.Message}");
        }
    }

    private void EnsureShortcut()
    {
        var executablePath = Environment.ProcessPath ?? Process.GetCurrentProcess().MainModule?.FileName;
        if (string.IsNullOrWhiteSpace(executablePath))
        {
            throw new InvalidOperationException("Cannot resolve the running executable path.");
        }

        Directory.CreateDirectory(Path.GetDirectoryName(shortcutPath)!);

        var shellLinkType = Type.GetTypeFromCLSID(ShellLinkClsid, throwOnError: true)!;
        var shellLinkObject = Activator.CreateInstance(shellLinkType)
            ?? throw new InvalidOperationException("Cannot create ShellLink COM object.");
        var shellLink = (IShellLinkW)shellLinkObject;
        shellLink.SetPath(executablePath);
        shellLink.SetArguments("");
        shellLink.SetWorkingDirectory(Path.GetDirectoryName(executablePath));
        shellLink.SetDescription("ToastDeck-A notification demo");

        var appIdKey = PropertyKeys.AppUserModelId;
        var appIdValue = PropVariant.FromString(AppId);
        var propertyStore = (IPropertyStore)shellLink;
        propertyStore.SetValue(ref appIdKey, ref appIdValue);
        propertyStore.Commit();
        appIdValue.Dispose();

        var persistFile = (IPersistFile)shellLink;
        persistFile.Save(shortcutPath, true);
    }

    private static readonly Guid ShellLinkClsid = new("00021401-0000-0000-C000-000000000046");

    [ComImport]
    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    [Guid("000214F9-0000-0000-C000-000000000046")]
    private interface IShellLinkW
    {
        void GetPath([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder pszFile, int cchMaxPath, IntPtr pfd, uint fFlags);
        void GetIDList(out IntPtr ppidl);
        void SetIDList(IntPtr pidl);
        void GetDescription([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder pszName, int cchMaxName);
        void SetDescription([MarshalAs(UnmanagedType.LPWStr)] string pszName);
        void GetWorkingDirectory([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder pszDir, int cchMaxPath);
        void SetWorkingDirectory([MarshalAs(UnmanagedType.LPWStr)] string? pszDir);
        void GetArguments([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder pszArgs, int cchMaxPath);
        void SetArguments([MarshalAs(UnmanagedType.LPWStr)] string pszArgs);
        void GetHotkey(out short pwHotkey);
        void SetHotkey(short wHotkey);
        void GetShowCmd(out int piShowCmd);
        void SetShowCmd(int iShowCmd);
        void GetIconLocation([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder pszIconPath, int cchIconPath, out int piIcon);
        void SetIconLocation([MarshalAs(UnmanagedType.LPWStr)] string pszIconPath, int iIcon);
        void SetRelativePath([MarshalAs(UnmanagedType.LPWStr)] string pszPathRel, uint dwReserved);
        void Resolve(IntPtr hwnd, uint fFlags);
        void SetPath([MarshalAs(UnmanagedType.LPWStr)] string pszFile);
    }

    [ComImport]
    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    [Guid("0000010b-0000-0000-C000-000000000046")]
    private interface IPersistFile
    {
        void GetClassID(out Guid pClassID);
        void IsDirty();
        void Load([MarshalAs(UnmanagedType.LPWStr)] string pszFileName, uint dwMode);
        void Save([MarshalAs(UnmanagedType.LPWStr)] string pszFileName, bool fRemember);
        void SaveCompleted([MarshalAs(UnmanagedType.LPWStr)] string pszFileName);
        void GetCurFile([MarshalAs(UnmanagedType.LPWStr)] out string ppszFileName);
    }

    [ComImport]
    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    [Guid("886D8EEB-8CF2-4446-8D02-CDBA1DBDCF99")]
    private interface IPropertyStore
    {
        void GetCount(out uint cProps);
        void GetAt(uint iProp, out PropertyKey pkey);
        void GetValue(ref PropertyKey key, out PropVariant pv);
        void SetValue(ref PropertyKey key, ref PropVariant pv);
        void Commit();
    }

    [StructLayout(LayoutKind.Sequential, Pack = 4)]
    private readonly struct PropertyKey
    {
        public PropertyKey(Guid formatId, uint propertyId)
        {
            FormatId = formatId;
            PropertyId = propertyId;
        }

        public Guid FormatId { get; }
        public uint PropertyId { get; }
    }

    private static class PropertyKeys
    {
        public static readonly PropertyKey AppUserModelId = new(new Guid("9F4C2855-9F79-4B39-A8D0-E1D42DE1D5F3"), 5);
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct PropVariant : IDisposable
    {
        private ushort vt;
        private ushort wReserved1;
        private ushort wReserved2;
        private ushort wReserved3;
        private IntPtr p;
        private IntPtr p2;

        public static PropVariant FromString(string value)
        {
            return new PropVariant
            {
                vt = 31,
                p = Marshal.StringToCoTaskMemUni(value)
            };
        }

        public void Dispose()
        {
            PropVariantClear(ref this);
        }

        [DllImport("ole32.dll")]
        private static extern int PropVariantClear(ref PropVariant pvar);
    }
}
