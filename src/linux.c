#include "linux.h"

struct WindowInfo get_window_info(Display *display)
{
    struct WindowInfo info;

    Window window;
    int revert;
    XGetInputFocus(display, &window, &revert);
    if (window == 1)
        return get_window_info(display);

    XWindowAttributes attr;
    Status status = XGetWindowAttributes(display, window, &attr);
    if (status == BadWindow)
        return info;

    XClassHint hint;
    status = XGetClassHint(display, window, &hint);
    if (!status)
        return info;

    info.res_class = hint.res_class;
    info.res_name = hint.res_name;

    XTextProperty text;
    status = XGetWMName(display, window, &text);
    if (status && text.value && text.nitems)
    {
        int i;
        char **list;
        status = XmbTextPropertyToTextList(display, &text, &list, &i);
        if (status >= Success && i && *list)
        {
            info.res_title = (char *)*list;
        }
    }

    return info;
}

void start_listening()
{
    Display *display = XOpenDisplay(NULL);
    Window root_window = DefaultRootWindow(display);

    XSelectInput(display, root_window, SubstructureNotifyMask);

    XEvent event;
    while (true)
    {
        XNextEvent(display, &event); // Wait for next event
        if (event.type == PropertyChangeMask || event.type == 22 || event.type == 18)
        {
            struct WindowInfo info = get_window_info(display);

            if (info.res_class)
                update_profile(info.res_class);

            if (info.res_name)
                update_profile(info.res_name);

            if (info.res_title)
                update_profile(info.res_title);
        }
    }
}

const char *get_config_path()
{
    return "";
}