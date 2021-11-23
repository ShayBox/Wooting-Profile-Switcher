#include "main.h"
#include <windows.h>
#include <winuser.h>
#include <psapi.h>
#include <synchapi.h>

void CALLBACK event_handler(HWINEVENTHOOK hook, DWORD event, HWND hwnd, LONG idObject, LONG idChild,
                            DWORD dwEventThread, DWORD dwmsEventTime);
void start_listening();
char *get_config_path();
void cleanup(int receivedSignal);
void usleep(unsigned int usec);