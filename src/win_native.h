#include "main.h"
#include <stdbool.h>
#include <windows.h>
#include <winuser.h>

void start_listening();
void CALLBACK event_handler(HWINEVENTHOOK hook, DWORD event, HWND hwnd, LONG idObject, LONG idChild, 
                                DWORD dwEventThread, DWORD dwmsEventTime);