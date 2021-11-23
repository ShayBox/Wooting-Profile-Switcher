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

struct WindowInfo last_info;
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
            int match_found = 0;
            if (info.res_class)
                if (last_info.res_class && strcmp(last_info.res_class, info.res_class))
                    match_found = update_profile(info.res_class);
            if (info.res_name && match_found == 0)
                if (last_info.res_name && strcmp(last_info.res_name, info.res_name))
                    match_found = update_profile(info.res_name);
            if (info.res_title && match_found == 0)
                if (last_info.res_title && strcmp(last_info.res_title, info.res_title))
                    update_profile(info.res_title);

            last_info = info;
        }
    }
}

const char *get_config_path()
{
    const char *xdg_config_home = getenv("XDG_CONFIG_HOME");
    if (xdg_config_home)
    {
        return strcat(xdg_config_home, "/WootingProfileSwitcher/config.json");
    }

    const char *home = getenv("HOME");
    if (home)
    {
        return strcat(home, "/.config/WootingProfileSwitcher/config.json");
    }

    return "./config.json";
}

void cleanup(void)
{
    reset_profile();
    wooting_rgb_reset();
    exit(EXIT_SUCCESS);
}