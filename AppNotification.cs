namespace ToastDesk;

public sealed class AppNotification
{
    public AppNotification(string title, string message, NotificationOrigin origin)
    {
        Id = Guid.NewGuid();
        Title = string.IsNullOrWhiteSpace(title) ? "Notification" : title.Trim();
        Message = string.IsNullOrWhiteSpace(message) ? "(No message body)" : message.Trim();
        Origin = origin;
        CreatedAt = DateTimeOffset.Now;
    }

    public Guid Id { get; }
    public string Title { get; }
    public string Message { get; }
    public NotificationOrigin Origin { get; }
    public DateTimeOffset CreatedAt { get; }
    public string CreatedAtLocalText => CreatedAt.ToString("HH:mm:ss");
}
