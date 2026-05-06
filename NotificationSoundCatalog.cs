using System.IO;

namespace ToastDesk;

public static class NotificationSoundCatalog
{
    public const string NonePresetId = "none";
    public const string CustomPresetId = "custom";
    public const string DefaultPresetId = "opencode-complete";

    private static readonly IReadOnlyList<NotificationSoundPreset> BuiltInPresets =
    [
        new("opencode-complete", "OpenCode Complete", "opencode-complete.wav"),
        new("opencode-permission", "OpenCode Permission", "opencode-permission.wav"),
        new("opencode-question", "OpenCode Question", "opencode-question.wav"),
        new("opencode-error", "OpenCode Error", "opencode-error.wav"),
        new("opencode-subagent-complete", "OpenCode Subagent", "opencode-subagent-complete.wav"),
        new("calm", "Calm", "Calm.wav"),
        new("glass", "Glass", "Glass.wav"),
        new("polite", "Polite", "Polite.wav"),
        new("glisten", "Glisten", "Glisten.wav"),
        new("chord", "Chord", "Chord.wav"),
        new("sharp", "Sharp", "Sharp.wav"),
        new("alarmed", "Alarmed", "Alarmed.wav")
    ];

    public static IReadOnlyList<NotificationSoundPreset> AllPresets { get; } =
    [
        ..BuiltInPresets,
        new(CustomPresetId, "Custom file", ""),
        new(NonePresetId, "Silent", "")
    ];

    public static string? ResolveSoundPath(AppSettings settings)
    {
        if (!settings.EnableNotificationSound || settings.DoNotDisturb || settings.SoundPresetId == NonePresetId)
        {
            return null;
        }

        if (settings.SoundPresetId == CustomPresetId)
        {
            return File.Exists(settings.CustomSoundPath) ? settings.CustomSoundPath : null;
        }

        var preset = BuiltInPresets.FirstOrDefault(item => item.Id == settings.SoundPresetId)
            ?? BuiltInPresets.First(item => item.Id == DefaultPresetId);
        var path = Path.Combine(AppContext.BaseDirectory, "assets", "sounds", preset.FileName);

        return File.Exists(path) ? path : null;
    }
}
