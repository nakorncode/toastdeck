using System.Windows.Threading;
using Windows.UI.Notifications;
using Windows.UI.Notifications.Management;

namespace ToastDeckA;

public sealed class WindowsNotificationListener : IDisposable
{
    private readonly NotificationStore store;
    private readonly Dispatcher dispatcher;
    private readonly HashSet<uint> capturedNotificationIds = [];
    private UserNotificationListener? listener;
    private bool isDisposed;

    public WindowsNotificationListener(NotificationStore store, Dispatcher dispatcher)
    {
        this.store = store;
        this.dispatcher = dispatcher;
    }

    public async Task<WindowsNotificationListenerResult> StartAsync()
    {
        if (isDisposed)
        {
            return new WindowsNotificationListenerResult(false, "Windows notification capture was cancelled because the app is closing.");
        }

        listener = UserNotificationListener.Current;
        var accessStatus = await listener.RequestAccessAsync();

        if (isDisposed)
        {
            return new WindowsNotificationListenerResult(false, "Windows notification capture was cancelled because the app is closing.");
        }

        if (accessStatus != UserNotificationListenerAccessStatus.Allowed)
        {
            return new WindowsNotificationListenerResult(
                false,
                $"Windows notification capture is not allowed. Status: {accessStatus}.");
        }

        listener.NotificationChanged -= OnNotificationChanged;
        listener.NotificationChanged += OnNotificationChanged;

        await CaptureCurrentNotificationsAsync();

        if (isDisposed)
        {
            return new WindowsNotificationListenerResult(false, "Windows notification capture was cancelled because the app is closing.");
        }

        return new WindowsNotificationListenerResult(
            true,
            "Windows notification capture is enabled. New Windows toasts will be mirrored into this demo list.");
    }

    public void Dispose()
    {
        if (isDisposed)
        {
            return;
        }

        isDisposed = true;

        if (listener is not null)
        {
            listener.NotificationChanged -= OnNotificationChanged;
            listener = null;
        }
    }

    private async void OnNotificationChanged(UserNotificationListener sender, UserNotificationChangedEventArgs args)
    {
        if (isDisposed || args.ChangeKind != UserNotificationChangedKind.Added)
        {
            return;
        }

        await CaptureCurrentNotificationsAsync();
    }

    private async Task CaptureCurrentNotificationsAsync()
    {
        if (isDisposed || listener is null)
        {
            return;
        }

        IReadOnlyList<UserNotification> notifications;

        try
        {
            notifications = await listener.GetNotificationsAsync(NotificationKinds.Toast);
        }
        catch (UnauthorizedAccessException)
        {
            return;
        }

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
}
