using System.IO;
using System.Runtime.ExceptionServices;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Windows;

namespace ToastDesk;

public static class CrashReporter
{
    private static int isShowingCrash;

    public static void Install(System.Windows.Application application)
    {
        application.DispatcherUnhandledException += (_, args) =>
        {
            ShowFatalException(args.Exception);
            args.Handled = true;
            application.Shutdown(-1);
        };

        AppDomain.CurrentDomain.UnhandledException += (_, args) =>
        {
            var exception = args.ExceptionObject as Exception
                ?? new InvalidOperationException($"Unhandled non-Exception object: {args.ExceptionObject}");
            ShowFatalException(exception);
        };

        TaskScheduler.UnobservedTaskException += (_, args) =>
        {
            ShowFatalException(args.Exception);
            args.SetObserved();
            application.Dispatcher.InvokeAsync(() => application.Shutdown(-1));
        };
    }

    public static string FormatExceptionSummary(Exception exception)
    {
        var exceptionInfo = ExceptionDispatchInfo.Capture(exception).SourceException;
        return $"{exceptionInfo.GetType().FullName} (HRESULT 0x{exceptionInfo.HResult:X8}): {exceptionInfo.Message}";
    }

    private static void ShowFatalException(Exception exception)
    {
        if (Interlocked.Exchange(ref isShowingCrash, 1) == 1)
        {
            return;
        }

        try
        {
            WriteCrashLog(exception);
            System.Windows.MessageBox.Show(
                BuildCrashMessage(exception),
                "ToastDesk crashed",
                MessageBoxButton.OK,
                MessageBoxImage.Error,
                MessageBoxResult.OK,
                System.Windows.MessageBoxOptions.DefaultDesktopOnly);
        }
        finally
        {
            Interlocked.Exchange(ref isShowingCrash, 0);
        }
    }

    private static string BuildCrashMessage(Exception exception)
    {
        var builder = new StringBuilder();
        builder.AppendLine("ToastDesk hit an unhandled error and must close.");
        builder.AppendLine();
        builder.AppendLine(FormatExceptionSummary(exception));
        builder.AppendLine();
        builder.AppendLine($"Source: {exception.Source ?? "Unknown"}");
        builder.AppendLine($"Target: {exception.TargetSite?.Name ?? "Unknown"}");
        builder.AppendLine();
        builder.AppendLine("Stack trace:");
        builder.AppendLine(exception.ToString());
        builder.AppendLine();
        builder.AppendLine($"A copy was written to: {GetCrashLogPath()}");
        return builder.ToString();
    }

    private static void WriteCrashLog(Exception exception)
    {
        var logPath = GetCrashLogPath();
        Directory.CreateDirectory(Path.GetDirectoryName(logPath)!);
        File.WriteAllText(
            logPath,
            $"{DateTimeOffset.Now:O}{Environment.NewLine}{BuildCrashMessage(exception)}");
    }

    private static string GetCrashLogPath()
    {
        return Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "ToastDesk",
            "crash-last.log");
    }
}
