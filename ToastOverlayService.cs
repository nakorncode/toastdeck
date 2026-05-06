using System.Windows;

namespace ToastDeckA;

public sealed class ToastOverlayService : IDisposable
{
    private const double ScreenMargin = 18;
    private const int MaxStackedToasts = 5;
    private const double StackOffset = 14;
    private const double StackScaleStep = 0.035;
    private const double StackOpacityStep = 0.14;
    private readonly NotificationStore store;
    private readonly Dictionary<Guid, ToastWindow> windows = [];
    private readonly List<Guid> displayOrder = [];

    public ToastOverlayService(NotificationStore store)
    {
        this.store = store;
        store.NotificationAdded += OnNotificationAdded;
        store.NotificationDismissed += OnNotificationDismissed;
    }

    public void Dispose()
    {
        store.NotificationAdded -= OnNotificationAdded;
        store.NotificationDismissed -= OnNotificationDismissed;

        foreach (var window in windows.Values.ToArray())
        {
            window.Close();
        }

        windows.Clear();
    }

    private void OnNotificationAdded(object? sender, AppNotification notification)
    {
        var window = new ToastWindow(notification, () => store.Dismiss(notification.Id));
        windows[notification.Id] = window;
        displayOrder.Insert(0, notification.Id);
        window.Show();
        ArrangeWindows();
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
        }
    }
}
