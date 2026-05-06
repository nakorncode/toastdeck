using System.Drawing;
using System.Windows;
using Forms = System.Windows.Forms;

namespace ToastDeckA;

public partial class App : System.Windows.Application
{
    private Forms.NotifyIcon? trayIcon;
    private MainWindow? mainWindow;
    private NotificationStore? store;
    private ToastOverlayService? overlayService;

    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);

        ShutdownMode = ShutdownMode.OnExplicitShutdown;

        store = new NotificationStore();
        overlayService = new ToastOverlayService(store);
        mainWindow = new MainWindow(store);
        mainWindow.Show();

        trayIcon = new Forms.NotifyIcon
        {
            Icon = SystemIcons.Application,
            Text = "ToastDeck-A",
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
        base.OnExit(e);
    }

    private Forms.ContextMenuStrip BuildTrayMenu()
    {
        var menu = new Forms.ContextMenuStrip();
        menu.Items.Add("Open", null, (_, _) => ShowMainWindow());
        menu.Items.Add("Test notification", null, (_, _) => store?.Add("Tray test", "Created from the background tray process.", NotificationOrigin.AppDemo));
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
