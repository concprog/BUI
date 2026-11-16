// System.Windows.Forms.Form: a top-level window over a BUI frame.
// SPDX-License-Identifier: GPL-2.0-or-later
using System.Drawing;
using Bui;

namespace Bui.Forms;

/// <summary>
/// A top-level window. Its controls live in a BUI frame; the native window is
/// made by <see cref="Show"/>, so a form can be built and laid out headless.
/// </summary>
public class Form : ContainerControl
{
    private readonly Context context;
    private Window? window;
    private string text = string.Empty;

    public Form()
        : this(new Frame())
    {
    }

    private Form(Frame frame)
        : base(frame, new Size(300, 300)) => context = new Context(frame); // the Windows Forms default

    /// <summary>The title bar text.</summary>
    public string Text
    {
        get => text;
        set
        {
            text = value;
            if (window is not null)
            {
                window.Title = value;
            }
        }
    }

    /// <summary>The kit context: input and layout without a window (tests, other hosts).</summary>
    public Context Context => context;

    /// <summary>Opens the window.</summary>
    public void Show()
    {
        window ??= new Window(context, text, Size.Width, Size.Height);
        window.Show();
    }

    /// <summary>Closes the window; <see cref="Application.Run"/> returns when the last one closes.</summary>
    public void Close() => window?.Close();

    protected override void Dispose(bool disposing)
    {
        if (disposing)
        {
            window?.Dispose();
            context.Dispose();
        }
        base.Dispose(disposing);
    }
}

/// <summary>Runs forms (System.Windows.Forms.Application).</summary>
public static class Application
{
    /// <summary>Shows <paramref name="form"/> and runs the message loop until every window is closed.</summary>
    public static void Run(Form form)
    {
        System.ArgumentNullException.ThrowIfNull(form);
        form.Show();
        Window.Run();
    }

    /// <summary>Stops <see cref="Run"/>.</summary>
    public static void Exit() => Window.Quit();
}
