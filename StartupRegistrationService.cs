using Microsoft.Win32;

namespace ToastDesk;

public sealed class StartupRegistrationService
{
    private const string RunKeyPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
    private const string ValueName = "ToastDesk";
    private const string LegacyValueName = "ToastDeck-A";

    public bool IsRegistered()
    {
        using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, writable: false);
        return key?.GetValue(ValueName) is string value && IsCurrentExecutable(value);
    }

    public void SetRegistered(bool isRegistered)
    {
        using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, writable: true)
            ?? Registry.CurrentUser.CreateSubKey(RunKeyPath, writable: true);

        if (!isRegistered)
        {
            key.DeleteValue(ValueName, throwOnMissingValue: false);
            key.DeleteValue(LegacyValueName, throwOnMissingValue: false);
            return;
        }

        var executablePath = Environment.ProcessPath;
        if (string.IsNullOrWhiteSpace(executablePath))
        {
            return;
        }

        key.SetValue(ValueName, $"\"{executablePath}\"");
        key.DeleteValue(LegacyValueName, throwOnMissingValue: false);
    }

    private static bool IsCurrentExecutable(string registeredValue)
    {
        var executablePath = Environment.ProcessPath;
        if (string.IsNullOrWhiteSpace(executablePath))
        {
            return false;
        }

        var normalizedRegisteredValue = registeredValue.Trim().Trim('"');
        return string.Equals(normalizedRegisteredValue, executablePath, StringComparison.OrdinalIgnoreCase);
    }
}
