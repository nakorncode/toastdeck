using System.Drawing;
using System.Windows;
using Forms = System.Windows.Forms;

namespace ToastDesk;

public partial class App : System.Windows.Application
{
    private SingleInstanceGuard? singleInstanceGuard;
    private Forms.NotifyIcon? trayIcon;
    private MainWindow? mainWindow;
    private NotificationStore? store;
    private ToastOverlayService? overlayService;
    private NotificationActionService? notificationActionService;
    private WindowsNotificationPlatform? notificationPlatform;
    private AppSettings? settings;

    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);

        singleInstanceGuard = new SingleInstanceGuard();
        if (!singleInstanceGuard.HasOwnership)
        {
            Shutdown();
            return;
        }

        ShutdownMode = ShutdownMode.OnExplicitShutdown;

        notificationPlatform = new WindowsNotificationPlatform();
        var platformResult = notificationPlatform.Initialize();

        var startupRegistrationService = new StartupRegistrationService();
        var settingsService = new AppSettingsService(startupRegistrationService);
        settings = settingsService.Load();

        store = new NotificationStore();
        notificationActionService = new NotificationActionService(ShowMainWindow);
        overlayService = new ToastOverlayService(store, settings, notificationActionService);
        mainWindow = new MainWindow(store, settings, notificationPlatform, platformResult);

        mainWindow.Show();
        if (settings.StartMinimized)
        {
            mainWindow.Hide();
        }

        trayIcon = new Forms.NotifyIcon
        {
            Icon = Icon.ExtractAssociatedIcon(Environment.ProcessPath ?? "") ?? SystemIcons.Application,
            Text = "ToastDesk",
            Visible = true,
            ContextMenuStrip = BuildTrayMenu()
        };
        trayIcon.DoubleClick += (_, _) => ShowMainWindow();
    }

    protected override void OnExit(ExitEventArgs e)
    {
        mainWindow?.Dispose();
        trayIcon?.Dispose();
        overlayService?.Dispose();
        singleInstanceGuard?.Dispose();
        base.OnExit(e);
    }

    private Forms.ContextMenuStrip BuildTrayMenu()
    {
        var menu = new Forms.ContextMenuStrip();
        menu.Items.Add("Open", null, (_, _) => ShowMainWindow());
        menu.Items.Add("Test notification", null, (_, _) => notificationPlatform?.SendTestNotification());
        menu.Items.Add("Exit", null, (_, _) => ExitApplication());
        return menu;
    }

    private void ExitApplication()
    {
        mainWindow?.AllowCloseForExit();
        Shutdown();
    }

    private void ShowMainWindow()
    {
        if (mainWindow is null)
        {
            return;
        }

        mainWindow.Show();
        mainWindow.WindowState = WindowState.Normal;
        mainWindow.Activate();
    }
}
