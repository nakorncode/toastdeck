using System.Windows;
using System.Windows.Interop;
using System.Windows.Media.Animation;

namespace ToastDeckA;

public partial class ToastWindow : Window
{
    private const int GwlExStyle = -20;
    private const int WsExNoActivate = 0x08000000;
    private static readonly Duration PlacementAnimationDuration = TimeSpan.FromMilliseconds(140);
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
}
