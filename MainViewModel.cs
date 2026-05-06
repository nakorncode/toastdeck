namespace ToastDeckA;

public sealed class MainViewModel
{
    public MainViewModel(NotificationStore notificationStore, AppSettings settings)
    {
        NotificationStore = notificationStore;
        Settings = settings;
    }

    public NotificationStore NotificationStore { get; }
    public AppSettings Settings { get; }
}
