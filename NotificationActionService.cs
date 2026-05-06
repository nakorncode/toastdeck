using System.Diagnostics;
using System.Windows;

namespace ToastDesk;

public sealed class NotificationActionService
{
    private readonly Action showToastDesk;

    public NotificationActionService(Action showToastDesk)
    {
        this.showToastDesk = showToastDesk;
    }

    public void Open(AppNotification notification)
    {
        if (TryOpenSourceApp(notification.SourceAppUserModelId))
        {
            return;
        }

        showToastDesk();
    }

    private static bool TryOpenSourceApp(string? appUserModelId)
    {
        if (string.IsNullOrWhiteSpace(appUserModelId))
        {
            return false;
        }

        try
        {
            Process.Start(new ProcessStartInfo
            {
                FileName = "explorer.exe",
                Arguments = $"shell:AppsFolder\\{appUserModelId}",
                UseShellExecute = true
            });
            return true;
        }
        catch (Exception ex) when (ex is InvalidOperationException or System.ComponentModel.Win32Exception)
        {
            return false;
        }
    }
}
