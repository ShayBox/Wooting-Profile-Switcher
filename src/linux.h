#include "main.h"
#include <X11/Xlib.h>
#include <X11/Xutil.h>

struct WindowInfo
{
    char *res_class;
    char *res_name;
};

struct WindowInfo get_window_info(Display *display);
void start_listening();
const char *get_config_path();