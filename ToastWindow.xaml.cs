using System.Windows;
using System.Windows.Interop;
using System.Windows.Media.Animation;

namespace ToastDesk;

public partial class ToastWindow : Window
{
    private const int GwlExStyle = -20;
    private const int WsExNoActivate = 0x08000000;
    private const uint SwpNoMove = 0x0002;
    private const uint SwpNoSize = 0x0001;
    private const uint SwpNoActivate = 0x0010;
    private static readonly IntPtr HwndTopmost = new(-1);
    private static readonly Duration PlacementAnimationDuration = TimeSpan.FromMilliseconds(140);
    private readonly Action open;
    private readonly Action dismiss;

    public ToastWindow(AppNotification notification, Action open, Action dismiss)
    {
        this.open = open;
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

    private void OpenButton_Click(object sender, RoutedEventArgs e)
    {
        open();
    }

    public void ApplyStackPlacement(double left, double top, double scale, double opacity, int stackIndex)
    {
        Topmost = true;
        Left = left;
        Top = top;
        Opacity = opacity;
        StackScaleTransform.BeginAnimation(
            System.Windows.Media.ScaleTransform.ScaleXProperty,
            new DoubleAnimation(scale, PlacementAnimationDuration));
        StackScaleTransform.BeginAnimation(
            System.Windows.Media.ScaleTransform.ScaleYProperty,
            new DoubleAnimation(scale, PlacementAnimationDuration));
    }

    public void RefreshTopmostOrder()
    {
        var handle = new WindowInteropHelper(this).Handle;
        NativeMethods.SetWindowPos(handle, HwndTopmost, 0, 0, 0, 0, SwpNoMove | SwpNoSize | SwpNoActivate);
    }
}
