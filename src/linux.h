#include "main.h"
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <unistd.h>

struct WindowInfo
{
    char *res_class;
    char *res_name;
    char *res_title;
};

struct WindowInfo get_window_info(Display *display);
void start_listening();
const char *get_config_path();
void cleanup(void);