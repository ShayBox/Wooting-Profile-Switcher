#include "linux.h"

struct WindowInfo get_window_hint(Display *display)
{
    struct WindowInfo hint;

    Window window;
    int revert;
    XGetInputFocus(display, &window, &revert);
    if (window == 1)
        return get_window_hint(display);

    XWindowAttributes attr;
    Status status = XGetWindowAttributes(display, window, &attr);
    if (status == BadWindow)
        return hint;

    XClassHint xhint;
    status = XGetClassHint(display, window, &xhint);
    if (!status)
        return hint;

    hint.class = xhint.res_class;
    hint.name = xhint.res_name;
    return hint;
}

void start_listening()
{
    Display *display = XOpenDisplay(NULL);
    Window root_window = DefaultRootWindow(display);

    XSelectInput(display, root_window, SubstructureNotifyMask);

    XEvent event;
    const char *old_class = "";
    const char *old_name = "";
    while (true)
    {
        XNextEvent(display, &event); // Wait for next event
        if (event.type == PropertyChangeMask || event.type == 22 || event.type == 18)
        {
            struct WindowInfo hint = get_window_hint(display);
            if (!hint.class || !hint.name)
                continue;

            if (strcmp(hint.class, old_class) || strcmp(hint.name, old_name))
            {
                old_class = hint.class;
                old_name = hint.name;
                update_profile(hint.name, hint.class);
            }
        }
    }
}