using System.Runtime.InteropServices;
using System.Threading;
using System.Windows.Threading;
using Windows.UI.Notifications;
using Windows.UI.Notifications.Management;

namespace ToastDesk;

public sealed class WindowsNotificationListener : IDisposable
{
    private static readonly TimeSpan EventBackupPollInterval = TimeSpan.FromSeconds(5);
    private static readonly TimeSpan FastPollInterval = TimeSpan.FromMilliseconds(500);
    private readonly NotificationStore store;
    private readonly Dispatcher dispatcher;
    private readonly HashSet<uint> capturedNotificationIds = [];
    private readonly DispatcherTimer syncTimer;
    private UserNotificationListener? listener;
    private int isCapturing;
    private bool isDisposed;
    private bool isEventSubscriptionEnabled;

    public string LastStatusMessage { get; private set; } = "Not started.";
    public bool IsEnabled { get; private set; }

    public WindowsNotificationListener(NotificationStore store, Dispatcher dispatcher)
    {
        this.store = store;
        this.dispatcher = dispatcher;
        syncTimer = new DispatcherTimer(DispatcherPriority.Background, dispatcher)
        {
            Interval = FastPollInterval
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
        ConfigureNotificationChangeMode();
        syncTimer.Start();

        if (isDisposed)
        {
            return SetStatus(false, "Windows notification capture was cancelled because the app is closing.");
        }

        return SetStatus(
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
        syncTimer.Stop();

        UnsubscribeNotificationChanged();
        listener = null;
    }

    public void Stop()
    {
        syncTimer.Stop();
        UnsubscribeNotificationChanged();
        listener = null;
        IsEnabled = false;
        LastStatusMessage = "Windows notification capture is disabled.";
    }

    private void ConfigureNotificationChangeMode()
    {
        UnsubscribeNotificationChanged();

        if (listener is null)
        {
            syncTimer.Interval = FastPollInterval;
            return;
        }

        try
        {
            listener.NotificationChanged += OnNotificationChanged;
            isEventSubscriptionEnabled = true;
            syncTimer.Interval = EventBackupPollInterval;
        }
        catch (COMException)
        {
            isEventSubscriptionEnabled = false;
            syncTimer.Interval = FastPollInterval;
        }
    }

    private void UnsubscribeNotificationChanged()
    {
        if (!isEventSubscriptionEnabled || listener is null)
        {
            isEventSubscriptionEnabled = false;
            return;
        }

        try
        {
            listener.NotificationChanged -= OnNotificationChanged;
        }
        catch (COMException)
        {
            // Ignore unsubscribe failures during shutdown or fallback transitions.
        }

        isEventSubscriptionEnabled = false;
    }

    private async void OnNotificationChanged(UserNotificationListener sender, UserNotificationChangedEventArgs args)
    {
        if (args.ChangeKind != UserNotificationChangedKind.Added)
        {
            return;
        }

        await CaptureCurrentNotificationsAsync();
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
        if (isDisposed || listener is null || Interlocked.Exchange(ref isCapturing, 1) == 1)
        {
            return;
        }

        try
        {
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

                var details = ExtractDetails(notification);
                _ = dispatcher.InvokeAsync(() =>
                {
                    if (!isDisposed)
                    {
                        store.Add(
                            details.Title,
                            details.Message,
                            NotificationOrigin.Windows,
                            details.SourceAppName,
                            details.SourceAppUserModelId);
                    }
                });
            }
        }
        finally
        {
            Interlocked.Exchange(ref isCapturing, 0);
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

    private static NotificationDetails ExtractDetails(UserNotification notification)
    {
        try
        {
            var binding = notification.Notification.Visual.GetBinding(KnownNotificationBindings.ToastGeneric);
            var textElements = binding?.GetTextElements().Select(item => item.Text).Where(text => !string.IsNullOrWhiteSpace(text)).ToArray() ?? [];
            var sourceAppName = notification.AppInfo?.DisplayInfo.DisplayName;
            var sourceAppUserModelId = notification.AppInfo?.AppUserModelId;

            var (title, message) = textElements.Length switch
            {
                0 => ("Windows notification", $"Notification ID {notification.Id}"),
                1 => (textElements[0], $"Notification ID {notification.Id}"),
                _ => (textElements[0], string.Join(Environment.NewLine, textElements.Skip(1)))
            };

            return new NotificationDetails(title, message, sourceAppName, sourceAppUserModelId);
        }
        catch (Exception ex) when (ex is COMException or InvalidOperationException or NotImplementedException or NullReferenceException)
        {
            return new NotificationDetails(
                "Unsupported Windows notification skipped",
                $"ToastDesk could not read notification ID {notification.Id}.{Environment.NewLine}{CrashReporter.FormatExceptionSummary(ex)}",
                null,
                null);
        }
    }

    private sealed record NotificationDetails(
        string Title,
        string Message,
        string? SourceAppName,
        string? SourceAppUserModelId);

    private WindowsNotificationListenerResult SetStatus(bool isEnabled, string message)
    {
        IsEnabled = isEnabled;
        LastStatusMessage = message;
        if (isEnabled && !string.IsNullOrWhiteSpace(message))
        {
            LastStatusMessage = isEventSubscriptionEnabled
                ? $"{message} Using Windows change events with polling backup."
                : $"{message} Using fast polling because Windows change events are unavailable.";
        }

        return new WindowsNotificationListenerResult(isEnabled, LastStatusMessage);
    }
}
