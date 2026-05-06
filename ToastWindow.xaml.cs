using System.Windows;
using System.Windows.Interop;

namespace ToastDeckA;

public partial class ToastWindow : Window
{
    private const int GwlExStyle = -20;
    private const int WsExNoActivate = 0x08000000;
    private readonly Action dismiss;

    public ToastWindow(AppNotification notification, Action dismiss)
    {
        this.dismiss = dismiss;
        InitializeComponent();
        DataContext = notification;
    }

    protected override void OnSourceInitialized(EventArgs e)
    {
        base.OnSourceInitialized(e);

        var handle = new WindowInteropHelper(this).Handle;
        var style = NativeMethods.GetWindowLong(handle, GwlExStyle);
        NativeMethods.SetWindowLong(handle, GwlExStyle, style | WsExNoActivate);
    }

    private void DismissButton_Click(object sender, RoutedEventArgs e)
    {
        dismiss();
    }
}
