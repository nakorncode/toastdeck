using System.ComponentModel;
using System.Runtime.CompilerServices;

namespace ToastDesk;

public sealed class AppSettings : INotifyPropertyChanged
{
    private bool startWithWindows = true;
    private bool startMinimized = true;
    private bool enableWindowsCapture = true;
    private bool enableToastOverlay = true;
    private bool enableNotificationSound = true;
    private bool doNotDisturb;
    private string soundPresetId = NotificationSoundCatalog.DefaultPresetId;
    private string? customSoundPath;
    private int notificationSoundVolume = 70;

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

    public bool EnableNotificationSound
    {
        get => enableNotificationSound;
        set => SetField(ref enableNotificationSound, value);
    }

    public bool DoNotDisturb
    {
        get => doNotDisturb;
        set => SetField(ref doNotDisturb, value);
    }

    public string SoundPresetId
    {
        get => soundPresetId;
        set => SetField(ref soundPresetId, value);
    }

    public string? CustomSoundPath
    {
        get => customSoundPath;
        set => SetField(ref customSoundPath, value);
    }

    public int NotificationSoundVolume
    {
        get => notificationSoundVolume;
        set
        {
            var clampedValue = Math.Clamp(value, 0, 100);
            SetField(ref notificationSoundVolume, clampedValue);
        }
    }

    private void SetField<T>(ref T field, T value, [CallerMemberName] string? propertyName = null)
    {
        if (EqualityComparer<T>.Default.Equals(field, value))
        {
            return;
        }

        field = value;
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }

}
