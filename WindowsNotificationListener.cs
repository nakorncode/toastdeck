using System.Runtime.InteropServices;
using System.Windows.Threading;
using Windows.UI.Notifications;
using Windows.UI.Notifications.Management;

namespace ToastDeckA;

public sealed class WindowsNotificationListener : IDisposable
{
    private readonly NotificationStore store;
    private readonly Dispatcher dispatcher;
    private readonly HashSet<uint> capturedNotificationIds = [];
    private readonly DispatcherTimer syncTimer;
    private UserNotificationListener? listener;
    private bool isDisposed;

    public string LastStatusMessage { get; private set; } = "Not started.";

    public WindowsNotificationListener(NotificationStore store, Dispatcher dispatcher)
    {
        this.store = store;
        this.dispatcher = dispatcher;
        syncTimer = new DispatcherTimer(DispatcherPriority.Background, dispatcher)
        {
            Interval = TimeSpan.FromSeconds(2)
        };
        syncTimer.Tick += async (_, _) => await CaptureCurrentNotificationsAsync();
    }

    public async Task<WindowsNotificationListenerResult> StartAsync(bool forcePrompt)
    {
        if (isDisposed)
        {
            return SetStatus(false, "Windows notification capture was cancelled because the app is closing.");
        }

        UserNotificationListenerAccessStatus accessStatus;
        try
        {
            listener = UserNotificationListener.Current;
            accessStatus = listener.GetAccessStatus();

            if (accessStatus == UserNotificationListenerAccessStatus.Unspecified || forcePrompt)
            {
                accessStatus = await listener.RequestAccessAsync();
            }
        }
        catch (COMException ex)
        {
            return SetStatus(false, $"Windows notification capture could not start: {ex.Message}");
        }

        if (isDisposed)
        {
            return SetStatus(false, "Windows notification capture was cancelled because the app is closing.");
        }

        if (accessStatus != UserNotificationListenerAccessStatus.Allowed)
        {
            return SetStatus(
                false,
                $"Windows notification capture is not allowed. Status: {accessStatus}. Use Retry Windows Capture, or allow notification access in Windows Settings if denied.");
        }

        await SeedExistingNotificationsAsync();
        syncTimer.Start();

        if (isDisposed)
        {
            return SetStatus(false, "Windows notification capture was cancelled because the app is closing.");
        }

        return SetStatus(
            true,
            "Windows notification capture is enabled. New Windows toasts will be mirrored into this demo list.");
    }

    public void ClearCapturedState()
    {
        capturedNotificationIds.Clear();
    }

    public void Dispose()
    {
        if (isDisposed)
        {
            return;
        }

        isDisposed = true;
        syncTimer.Stop();

        listener = null;
    }

    private async Task SeedExistingNotificationsAsync()
    {
        if (isDisposed || listener is null)
        {
            return;
        }

        var notifications = await GetCurrentNotificationsAsync();
        foreach (var notification in notifications)
        {
            capturedNotificationIds.Add(notification.Id);
        }
    }

    private async Task CaptureCurrentNotificationsAsync()
    {
        if (isDisposed || listener is null)
        {
            return;
        }

        var notifications = await GetCurrentNotificationsAsync();

        foreach (var notification in notifications)
        {
            if (isDisposed || dispatcher.HasShutdownStarted)
            {
                return;
            }

            if (!capturedNotificationIds.Add(notification.Id))
            {
                continue;
            }

            var (title, message) = ExtractText(notification);
            _ = dispatcher.InvokeAsync(() =>
            {
                if (!isDisposed)
                {
                    store.Add(title, message, NotificationOrigin.Windows);
                }
            });
        }
    }

    private async Task<IReadOnlyList<UserNotification>> GetCurrentNotificationsAsync()
    {
        if (listener is null)
        {
            return [];
        }

        try
        {
            return await listener.GetNotificationsAsync(NotificationKinds.Toast);
        }
        catch (COMException)
        {
            return [];
        }
        catch (UnauthorizedAccessException)
        {
            return [];
        }
    }

    private static (string Title, string Message) ExtractText(UserNotification notification)
    {
        var binding = notification.Notification.Visual.GetBinding(KnownNotificationBindings.ToastGeneric);
        var textElements = binding?.GetTextElements().Select(item => item.Text).Where(text => !string.IsNullOrWhiteSpace(text)).ToArray() ?? [];

        return textElements.Length switch
        {
            0 => ("Windows notification", $"Notification ID {notification.Id}"),
            1 => (textElements[0], $"Notification ID {notification.Id}"),
            _ => (textElements[0], string.Join(Environment.NewLine, textElements.Skip(1)))
        };
    }

    private WindowsNotificationListenerResult SetStatus(bool isEnabled, string message)
    {
        LastStatusMessage = message;
        return new WindowsNotificationListenerResult(isEnabled, message);
    }
}
