#include "main.h"
#include <Carbon/Carbon.h>
#include <Cocoa/Cocoa.h>
#include <AppKit/AppKit.h>
#include <unistd.h>
#include <sys/stat.h>

int init_window(void);
void start_listening(void);
void run(void);
void append_text_to_view(char* text);
void append_error_to_view(char* text);
const char *get_config_path(void);
void cleanup(int receivedSignal);
