using System.Collections.ObjectModel;
namespace ToastDeckA;

public sealed class NotificationStore
{
    public ObservableCollection<AppNotification> Notifications { get; } = [];

    public event EventHandler<AppNotification>? NotificationAdded;
    public event EventHandler<Guid>? NotificationDismissed;

    public AppNotification Add(string title, string message, NotificationOrigin origin)
    {
        var notification = new AppNotification(title, message, origin);

        if (System.Windows.Application.Current.Dispatcher.CheckAccess())
        {
            AddOnUiThread(notification);
        }
        else
        {
            System.Windows.Application.Current.Dispatcher.Invoke(() => AddOnUiThread(notification));
        }

        return notification;
    }

    public void Dismiss(Guid id)
    {
        var existing = Notifications.FirstOrDefault(item => item.Id == id);
        if (existing is not null)
        {
            Notifications.Remove(existing);
        }

        NotificationDismissed?.Invoke(this, id);
    }

    public void Clear()
    {
        var ids = Notifications.Select(item => item.Id).ToArray();
        Notifications.Clear();

        foreach (var id in ids)
        {
            NotificationDismissed?.Invoke(this, id);
        }
    }

    private void AddOnUiThread(AppNotification notification)
    {
        Notifications.Insert(0, notification);
        NotificationAdded?.Invoke(this, notification);
    }
}
