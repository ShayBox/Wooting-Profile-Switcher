#include "main.h"
#include <Carbon/Carbon.h>
#include <Cocoa/Cocoa.h>
#include <AppKit/AppKit.h>
#include <unistd.h>
#include <sys/stat.h>

void start_listening(void);
void run(void);
const char *get_config_path(void);
void cleanup(int receivedSignal);
