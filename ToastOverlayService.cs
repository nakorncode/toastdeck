using System.Windows;

namespace ToastDeckA;

public sealed class ToastOverlayService : IDisposable
{
    private const double ToastSpacing = 12;
    private const double ScreenMargin = 18;
    private readonly NotificationStore store;
    private readonly Dictionary<Guid, ToastWindow> windows = [];

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
        ArrangeWindows();
    }

    private void ArrangeWindows()
    {
        var workArea = SystemParameters.WorkArea;
        var top = workArea.Top + ScreenMargin;
        var right = workArea.Right - ScreenMargin;

        foreach (var window in windows.Values)
        {
            window.Left = right - window.Width;
            window.Top = top;
            top += window.Height + ToastSpacing;
        }
    }
}
