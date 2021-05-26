#include "win_native.h"

HWINEVENTHOOK event_hook;
char *old_name = "";

void initialize_event_hook()
{
    event_hook = SetWinEventHook(
        EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND,  	// Event Range to handle
        NULL,
        event_handler,                                      // callback function
        0, 0,                                               // process and thread IDs of interest (0 = all)
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
    LPSTR proc_path = malloc(2048);
    GetWindowText(hwnd, title, 256);

    DWORD tid, pid;
    tid = GetWindowThreadProcessId(hwnd, &pid);

    HANDLE hProc = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ , FALSE, pid);    
    GetModuleFileNameEx(hProc, NULL, proc_path, 2048);

    LPSTR proc_title = last_occurence((char*)proc_path, '\\')+1;

    CloseHandle(hProc);

#ifdef _DEBUG
    printf("%s, %d, %s, %d\n", title, tid, proc_title, pid);
#endif

    free(title);
    free(proc_path);
    free(&pid);
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

char* last_occurence(char* str, char chr)
{
    int i, index;
    for(i = 0; i <= strlen(str); i++)
    {
        if(str[i] == chr)
        {
            index = i;
        }
    }
    return &str[index];
}