namespace ToastDesk;

public sealed class AppNotification
{
    public AppNotification(
        string title,
        string message,
        NotificationOrigin origin,
        string? sourceAppName = null,
        string? sourceAppUserModelId = null)
    {
        Id = Guid.NewGuid();
        Title = string.IsNullOrWhiteSpace(title) ? "Notification" : title.Trim();
        Message = string.IsNullOrWhiteSpace(message) ? "(No message body)" : message.Trim();
        Origin = origin;
        SourceAppName = string.IsNullOrWhiteSpace(sourceAppName) ? null : sourceAppName.Trim();
        SourceAppUserModelId = string.IsNullOrWhiteSpace(sourceAppUserModelId) ? null : sourceAppUserModelId.Trim();
        CreatedAt = DateTimeOffset.Now;
    }

    public Guid Id { get; }
    public string Title { get; }
    public string Message { get; }
    public NotificationOrigin Origin { get; }
    public string? SourceAppName { get; }
    public string? SourceAppUserModelId { get; }
    public string SourceDisplayName => SourceAppName ?? Origin.ToString();
    public DateTimeOffset CreatedAt { get; }
    public string CreatedAtLocalText => CreatedAt.ToString("HH:mm:ss");
}
