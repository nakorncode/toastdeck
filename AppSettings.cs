using System.ComponentModel;
using System.Runtime.CompilerServices;

namespace ToastDeckA;

public sealed class AppSettings : INotifyPropertyChanged
{
    private bool startWithWindows = true;
    private bool startMinimized = true;
    private bool enableWindowsCapture = true;
    private bool enableToastOverlay = true;
    private bool doNotDisturb;

    public event PropertyChangedEventHandler? PropertyChanged;

    public bool StartWithWindows
    {
        get => startWithWindows;
        set => SetField(ref startWithWindows, value);
    }

    public bool StartMinimized
    {
        get => startMinimized;
        set => SetField(ref startMinimized, value);
    }

    public bool EnableWindowsCapture
    {
        get => enableWindowsCapture;
        set => SetField(ref enableWindowsCapture, value);
    }

    public bool EnableToastOverlay
    {
        get => enableToastOverlay;
        set => SetField(ref enableToastOverlay, value);
    }

    public bool DoNotDisturb
    {
        get => doNotDisturb;
        set => SetField(ref doNotDisturb, value);
    }

    private void SetField(ref bool field, bool value, [CallerMemberName] string? propertyName = null)
    {
        if (field == value)
        {
            return;
        }

        field = value;
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }
}
