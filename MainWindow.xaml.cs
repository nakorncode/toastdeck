using System.Windows;

namespace ToastDeckA;

public partial class MainWindow : Window, IDisposable
{
    private readonly WindowsNotificationPlatform notificationPlatform;
    private readonly WindowsNotificationListener windowsNotificationListener;
    private readonly WindowsNotificationPlatformResult platformResult;
    private bool allowClose;
    private bool isDisposed;

    public MainWindow(
        NotificationStore store,
        WindowsNotificationPlatform notificationPlatform,
        WindowsNotificationPlatformResult platformResult)
    {
        this.notificationPlatform = notificationPlatform;
        this.platformResult = platformResult;
        windowsNotificationListener = new WindowsNotificationListener(store, Dispatcher);

        InitializeComponent();
        DataContext = store;
        Loaded += MainWindow_Loaded;
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
        if (allowClose)
        {
            Dispose();
            return;
        }

        e.Cancel = true;
        Hide();
    }

    public void AllowCloseForExit()
    {
        allowClose = true;
    }

    public void Dispose()
    {
        if (isDisposed)
        {
            return;
        }

        isDisposed = true;
        windowsNotificationListener.Dispose();
    }

    private void TestNotificationButton_Click(object sender, RoutedEventArgs e)
    {
        var result = notificationPlatform.SendTestNotification();
        ListenerStatusText.Text = $"{result.Message} Capture status: {windowsNotificationListener.LastStatusMessage}";
    }

    private async void EnableWindowsListenerButton_Click(object sender, RoutedEventArgs e)
    {
        await StartWindowsCaptureAsync(forcePrompt: true);
    }

    private void ClearNotificationsButton_Click(object sender, RoutedEventArgs e)
    {
        ((NotificationStore)DataContext).Clear();
        ListenerStatusText.Text = $"{platformResult.Message} Cleared ToastDeck's visible list. Windows capture baseline is preserved.";
    }

    private async void MainWindow_Loaded(object sender, RoutedEventArgs e)
    {
        Loaded -= MainWindow_Loaded;
        await StartWindowsCaptureAsync(forcePrompt: false);
    }

    private async Task StartWindowsCaptureAsync(bool forcePrompt)
    {
        EnableWindowsListenerButton.IsEnabled = false;
        ListenerStatusText.Text = $"{platformResult.Message} Starting Windows notification capture...";

        var result = await windowsNotificationListener.StartAsync(forcePrompt);

        if (isDisposed || !IsLoaded)
        {
            return;
        }

        ListenerStatusText.Text = $"{platformResult.Message} {result.Message}";
        EnableWindowsListenerButton.IsEnabled = !result.IsEnabled;
    }
}
