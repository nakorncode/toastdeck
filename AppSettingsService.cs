using System.ComponentModel;
using System.IO;
using System.Text.Json;

namespace ToastDeckA;

public sealed class AppSettingsService
{
    private readonly string settingsPath;
    private readonly StartupRegistrationService startupRegistrationService;
    private bool isSaving;

    public AppSettingsService(StartupRegistrationService startupRegistrationService)
    {
        this.startupRegistrationService = startupRegistrationService;
        var settingsDir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "ToastDeck-A");
        settingsPath = Path.Combine(settingsDir, "settings.json");
    }

    public AppSettings Load()
    {
        var settings = LoadFromDisk();
        settings.StartWithWindows = startupRegistrationService.IsRegistered();
        settings.PropertyChanged += OnSettingsChanged;
        return settings;
    }

    private AppSettings LoadFromDisk()
    {
        try
        {
            if (!File.Exists(settingsPath))
            {
                return new AppSettings();
            }

            return JsonSerializer.Deserialize<AppSettings>(File.ReadAllText(settingsPath)) ?? new AppSettings();
        }
        catch (JsonException)
        {
            return new AppSettings();
        }
        catch (IOException)
        {
            return new AppSettings();
        }
    }

    private void OnSettingsChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (sender is not AppSettings settings || isSaving)
        {
            return;
        }

        if (e.PropertyName == nameof(AppSettings.StartWithWindows))
        {
            startupRegistrationService.SetRegistered(settings.StartWithWindows);
        }

        Save(settings);
    }

    private void Save(AppSettings settings)
    {
        try
        {
            isSaving = true;
            Directory.CreateDirectory(Path.GetDirectoryName(settingsPath)!);
            var json = JsonSerializer.Serialize(settings, new JsonSerializerOptions { WriteIndented = true });
            File.WriteAllText(settingsPath, json);
        }
        finally
        {
            isSaving = false;
        }
    }
}
