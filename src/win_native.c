#include "win_native.h"

HWINEVENTHOOK event_hook;
bool terminating = false;

LPSTR old_title[256];
LPSTR old_proc_title[2048];

void initialize_event_hook()
{
    event_hook = SetWinEventHook(
        EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, // Event Range to handle
        NULL,
        event_handler, // callback function
        0, 0,          // process and thread IDs of interest (0 = all)
        0              // flags
    );
}

void CALLBACK event_handler(HWINEVENTHOOK hook, DWORD event, HWND hwnd, LONG idObject, LONG idChild,
                            DWORD dwEventThread, DWORD dwmsEventTime)
{
    if (terminating) return;
    LPSTR title = malloc(256);
    LPSTR proc_path = malloc(2048);
    GetWindowText(hwnd, title, 256);

    DWORD tid, pid;
    tid = GetWindowThreadProcessId(hwnd, &pid);

    HANDLE hProc = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, pid);
    GetModuleFileNameEx(hProc, NULL, proc_path, 2048);

    LPSTR proc_title = strrchr((char *)proc_path, '\\') + 1;

    CloseHandle(hProc);

#ifdef _DEBUG
    printf("%s, %d, %s, %d\n", title, tid, proc_title, pid);
#endif

    if (strcmp((const char *)old_title, (const char *)title) || strcmp((const char *)old_proc_title, (const char *)proc_title))
    {
        strcpy((char *)old_title, (const char *)title);
        strcpy((char *)old_proc_title, (const char *)proc_title);

        int match_found = 0;

        match_found = update_profile(title);

        if (match_found == 0)
            update_profile(proc_title);
    }

    free(title);
    free(proc_path);
}

void start_listening()
{
    SetConsoleTitle(TEXT("Wooting Profile Switcher"));
    initialize_event_hook();

    MSG msg;
    while (GetMessage(&msg, NULL, 0, 0) > 0)
    {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
}

const char *get_config_path()
{
    return "";
}

void cleanup(void)
{
    terminating = true;
    UnhookWindowsHookEx(event_hook);
    wooting_rgb_reset();
    exit(EXIT_SUCCESS);
}