using System.Windows.Media;

namespace ToastDesk;

public sealed class NotificationSoundService : IDisposable
{
    private readonly NotificationStore store;
    private readonly AppSettings settings;
    private readonly MediaPlayer player = new();

    public NotificationSoundService(NotificationStore store, AppSettings settings)
    {
        this.store = store;
        this.settings = settings;
        store.NotificationAdded += OnNotificationAdded;
    }

    public void Dispose()
    {
        store.NotificationAdded -= OnNotificationAdded;
        player.Close();
    }

    public bool PlayPreview()
    {
        return PlayConfiguredSound();
    }

    private void OnNotificationAdded(object? sender, AppNotification notification)
    {
        PlayConfiguredSound();
    }

    private bool PlayConfiguredSound()
    {
        var soundPath = NotificationSoundCatalog.ResolveSoundPath(settings);
        if (string.IsNullOrWhiteSpace(soundPath))
        {
            return false;
        }

        player.Stop();
        player.Open(new Uri(soundPath, UriKind.Absolute));
        player.Volume = Math.Clamp(settings.NotificationSoundVolume / 100.0, 0, 1);
        player.Play();
        return true;
    }
}
