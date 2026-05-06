using System.Threading;

namespace ToastDesk;

public sealed class SingleInstanceGuard : IDisposable
{
    private const string MutexName = @"Local\NakornCode.ToastDesk";
    private readonly Mutex mutex;

    public SingleInstanceGuard()
    {
        mutex = new Mutex(initiallyOwned: true, MutexName, out var createdNew);
        HasOwnership = createdNew;
    }

    public bool HasOwnership { get; }

    public void Dispose()
    {
        if (HasOwnership)
        {
            mutex.ReleaseMutex();
        }

        mutex.Dispose();
    }
}
