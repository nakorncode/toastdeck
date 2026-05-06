using System.Windows;
using System.ComponentModel;

namespace ToastDeckA;

public partial class MainWindow : Window, IDisposable
{
    private readonly WindowsNotificationPlatform notificationPlatform;
    private readonly WindowsNotificationListener windowsNotificationListener;
    private readonly WindowsNotificationPlatformResult platformResult;
    private readonly AppSettings settings;
    private readonly NotificationStore store;
    private bool allowClose;
    private bool isDisposed;

    public MainWindow(
        NotificationStore store,
        AppSettings settings,
        WindowsNotificationPlatform notificationPlatform,
        WindowsNotificationPlatformResult platformResult)
    {
        this.store = store;
        this.settings = settings;
        this.notificationPlatform = notificationPlatform;
        this.platformResult = platformResult;
        windowsNotificationListener = new WindowsNotificationListener(store, Dispatcher);

        InitializeComponent();
        DataContext = new MainViewModel(store, settings);
        Loaded += MainWindow_Loaded;
        settings.PropertyChanged += Settings_PropertyChanged;
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
        settings.PropertyChanged -= Settings_PropertyChanged;
        windowsNotificationListener.Dispose();
    }

    private void TestNotificationButton_Click(object sender, RoutedEventArgs e)
    {
        var result = notificationPlatform.SendTestNotification();
        ListenerStatusText.Text = $"{result.Message} Capture status: {windowsNotificationListener.LastStatusMessage}";
    }

    private async void EnableWindowsListenerButton_Click(object sender, RoutedEventArgs e)
    {
        if (!settings.EnableWindowsCapture)
        {
            settings.EnableWindowsCapture = true;
            return;
        }

        await StartWindowsCaptureAsync(forcePrompt: true);
    }

    private void ClearNotificationsButton_Click(object sender, RoutedEventArgs e)
    {
        store.Clear();
        ListenerStatusText.Text = $"{platformResult.Message} Cleared ToastDeck's visible list. Windows capture baseline is preserved.";
    }

    private async void MainWindow_Loaded(object sender, RoutedEventArgs e)
    {
        Loaded -= MainWindow_Loaded;
        if (settings.EnableWindowsCapture)
        {
            await StartWindowsCaptureAsync(forcePrompt: false);
        }
        else
        {
            ListenerStatusText.Text = $"{platformResult.Message} Windows capture is disabled in Settings.";
            EnableWindowsListenerButton.IsEnabled = true;
        }
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

    private async void Settings_PropertyChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (isDisposed || e.PropertyName != nameof(AppSettings.EnableWindowsCapture))
        {
            return;
        }

        if (settings.EnableWindowsCapture)
        {
            await StartWindowsCaptureAsync(forcePrompt: false);
            return;
        }

        windowsNotificationListener.Stop();
        ListenerStatusText.Text = $"{platformResult.Message} Windows capture is disabled in Settings.";
        EnableWindowsListenerButton.IsEnabled = true;
    }
}
