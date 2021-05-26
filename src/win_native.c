#include "win_native.h"

HWINEVENTHOOK event_hook;
char *old_name = "";

void initialize_event_hook()
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
    LPSTR title = malloc(256);
    GetWindowTextA(hwnd, title, 256);

    DWORD tid, pid;
    tid = GetWindowThreadProcessId(hwnd, &pid);

#ifdef _DEBUG
    printf("%s, %d, %d\n", title, tid, pid);
#endif
}

void start_listening()
{
    initialize_event_hook();

    MSG msg;
    while(GetMessage(&msg, NULL, 0, 0) > 0)
    {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
}