using System.Windows;

namespace ToastDesk;

public sealed class ToastOverlayService : IDisposable
{
    private const double ScreenMargin = 18;
    private const int MaxStackedToasts = 4;
    private const double StackOffset = 12;
    private const double StackScaleStep = 0.04;
    private const double StackOpacityStep = 0.16;
    private readonly NotificationStore store;
    private readonly AppSettings settings;
    private readonly NotificationActionService notificationActionService;
    private readonly Dictionary<Guid, ToastWindow> windows = [];
    private readonly List<Guid> displayOrder = [];

    public ToastOverlayService(
        NotificationStore store,
        AppSettings settings,
        NotificationActionService notificationActionService)
    {
        this.store = store;
        this.settings = settings;
        this.notificationActionService = notificationActionService;
        store.NotificationAdded += OnNotificationAdded;
        store.NotificationDismissed += OnNotificationDismissed;
        settings.PropertyChanged += OnSettingsChanged;
    }

    public void Dispose()
    {
        store.NotificationAdded -= OnNotificationAdded;
        store.NotificationDismissed -= OnNotificationDismissed;
        settings.PropertyChanged -= OnSettingsChanged;

        foreach (var window in windows.Values.ToArray())
        {
            window.Close();
        }

        windows.Clear();
    }

    private void OnNotificationAdded(object? sender, AppNotification notification)
    {
        if (!settings.EnableToastOverlay || settings.DoNotDisturb)
        {
            return;
        }

        var window = new ToastWindow(
            notification,
            () =>
            {
                notificationActionService.Open(notification);
                store.Dismiss(notification.Id);
            },
            () => store.Dismiss(notification.Id));
        windows[notification.Id] = window;
        displayOrder.Insert(0, notification.Id);
        window.Show();
        ArrangeWindows();
    }

    private void OnSettingsChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
    {
        if (e.PropertyName is nameof(AppSettings.EnableToastOverlay) or nameof(AppSettings.DoNotDisturb))
        {
            if (!settings.EnableToastOverlay || settings.DoNotDisturb)
            {
                CloseAllWindows();
            }
        }
    }

    private void CloseAllWindows()
    {
        foreach (var window in windows.Values.ToArray())
        {
            window.Close();
        }

        windows.Clear();
        displayOrder.Clear();
    }

    private void OnNotificationDismissed(object? sender, Guid id)
    {
        if (!windows.Remove(id, out var window))
        {
            return;
        }

        window.Close();
        displayOrder.Remove(id);
        ArrangeWindows();
    }

    private void ArrangeWindows()
    {
        var workArea = SystemParameters.WorkArea;
        var baseTop = workArea.Top + ScreenMargin;
        var right = workArea.Right - ScreenMargin;
        var visibleWindows = new List<ToastWindow>();

        for (var index = 0; index < displayOrder.Count; index++)
        {
            if (!windows.TryGetValue(displayOrder[index], out var window))
            {
                continue;
            }

            if (index >= MaxStackedToasts)
            {
                window.Hide();
                continue;
            }

            var scale = Math.Max(0.84, 1 - (index * StackScaleStep));
            var opacity = Math.Max(0.42, 1 - (index * StackOpacityStep));
            var top = baseTop + (index * StackOffset);
            var left = right - (window.Width * scale) - (index * 3);

            if (!window.IsVisible)
            {
                window.Show();
            }

            window.ApplyStackPlacement(left, top, scale, opacity, index);
            visibleWindows.Add(window);
        }

        for (var index = visibleWindows.Count - 1; index >= 0; index--)
        {
            visibleWindows[index].RefreshTopmostOrder();
        }
    }
}
