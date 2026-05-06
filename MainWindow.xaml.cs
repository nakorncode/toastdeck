using System.Windows;

namespace ToastDeckA;

public partial class MainWindow : Window
{
    private readonly NotificationStore store;
    private readonly WindowsNotificationListener windowsNotificationListener;

    public MainWindow(NotificationStore store)
    {
        this.store = store;
        windowsNotificationListener = new WindowsNotificationListener(store, Dispatcher);

        InitializeComponent();
        DataContext = store;
    }

    protected override void OnStateChanged(EventArgs e)
    {
        base.OnStateChanged(e);

        if (WindowState == WindowState.Minimized)
        {
            Hide();
        }
    }

    protected override void OnClosing(System.ComponentModel.CancelEventArgs e)
    {
        e.Cancel = true;
        Hide();
    }

    private void TestNotificationButton_Click(object sender, RoutedEventArgs e)
    {
        var count = store.Notifications.Count + 1;
        store.Add(
            $"Demo notification {count}",
            "This persistent toast stays visible until an action is clicked.",
            NotificationOrigin.AppDemo);
    }

    private async void EnableWindowsListenerButton_Click(object sender, RoutedEventArgs e)
    {
        EnableWindowsListenerButton.IsEnabled = false;
        ListenerStatusText.Text = "Requesting Windows notification capture permission...";

        var result = await windowsNotificationListener.StartAsync();
        ListenerStatusText.Text = result.Message;
        EnableWindowsListenerButton.IsEnabled = !result.IsEnabled;
    }

    private void ClearNotificationsButton_Click(object sender, RoutedEventArgs e)
    {
        store.Clear();
    }
}
