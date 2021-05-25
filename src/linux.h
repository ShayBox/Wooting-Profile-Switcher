#include <X11/Xlib.h>

struct WindowInfo
{
    char *class;
    char *name;
};

struct WindowInfo get_window_hint(Display *display);
void start_listening();