#include "windows.h"

HWINEVENTHOOK event_hook;
char *old_name = "";

void initilize_event_hook()
{
    event_hook = SetWinEventHook(
        EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND,  	// Event Range to handle
        NULL,
        event_handler,										// callback function
        0, 0,              									// process and thread IDs of interest (0 = all)
        0 	                                                // flags
    );
}

void cleanup()
{
    UnhookWinEvent(event_hook);
}

void CALLBACK event_handler(HWINEVENTHOOK hook, DWORD event, HWND hwnd, LONG idObject, LONG idChild, 
                                DWORD dwEventThread, DWORD dwmsEventTime)
{
    LPSTR title;
    GetWindowTextA(hwnd, title, 256);

    DWORD tid, pid;
    tid = GetWindowThreadProcessId(hwnd, &pid);

#ifdef DEBUG
    printf("%s, %d, %d", title, tid, pid);
#endif
}

void start_listening()
{
    initilize_event_hook();
}