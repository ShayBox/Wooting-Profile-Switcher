#include "main.h"
#include <stdbool.h>
#include <string.h>
#include <X11/Xlib.h>
#include <X11/Xutil.h>

const char *get_window_name(Display *display)
{
    Window window;
    int revert;
    XGetInputFocus(display, &window, &revert);
    if (window == 1)
        return get_window_name(display);

    XWindowAttributes attr;
    Status status = XGetWindowAttributes(display, window, &attr);
    if (status == BadWindow)
        return NULL;

    XClassHint hint;
    status = XGetClassHint(display, window, &hint);
    if (!status)
        return NULL;

    return hint.res_name;
}

void start_listening()
{
    Display *display = XOpenDisplay(NULL);
    Window root_window = DefaultRootWindow(display);

    XSelectInput(display, root_window, SubstructureNotifyMask);

    XEvent event;
    const char *old_name = "";
    while (true)
    {
        XNextEvent(display, &event); // Wait for next event
        if (event.type == PropertyChangeMask || event.type == 22 || event.type == 18)
        {
            const char *new_name = get_window_name(display);
            if (!new_name)
                continue;

            if (strcmp(new_name, old_name))
            {
                old_name = new_name;
                update_profile(new_name);
            }
        }
    }
}