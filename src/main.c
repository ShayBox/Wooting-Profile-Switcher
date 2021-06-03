#ifdef _WIN32
#include "win_native.h"
#elif __APPLE__
#include "mac.h"
#elif __linux__
#include "linux.h"
#endif
#include "main.h"

// https://gist.github.com/BigBrainAFK/0ba454a1efb43f7cb6301cda8838f432
const char ReloadProfile0 = 7;
const char GetCurrentKeyboardProfileIndex = 11;
const char ActivateProfile = 23;
const char RefreshRgbColors = 29;
const char WootDevResetAll = 32;

struct Process
{
    const char *match;
    const int profile;
};

// TODO: Add config file
struct Process process_list[] = {
#ifdef _WIN32
    {"Isaac", 1},
    {"isaac-ng.exe", 2},
#elif __APPLE__
    {"", 0},
#elif __linux__
    {"steam_app_250900", 1}, // Binding of Isaac
#endif
};

int main()
{
    if (!wooting_rgb_kbd_connected())
    {
        puts("Keyboard not connected.");
        return EXIT_FAILURE;
    }

    uint8_t buff[256] = {0};

#ifdef _DEBUG
    printf("%d\n", buff[0]);
#endif

    int read_result = wooting_usb_send_feature_with_response(buff, 256, GetCurrentKeyboardProfileIndex, 0, 0, 0, 0);

#ifdef _DEBUG
    printf("Bytes read: %d\n", read_result);

    printf("Buffer \n");
    for(int i = 0; i < 256; i++ )
    {
        printf("%d%s", buff[i], i < 255 ? ", " : "");
    }
    printf("\n");
#endif

    // process_list = // TODO: Add config

    // Exit handler so the platforms can clean up their hooks if necessary
    register_cleanup();

    start_listening();
}

int last_profile = -1;
const char *last_match = "";
int update_profile(const char *match)
{
    int match_found = 0;

    if (strcmp(match, last_match) == 0)
        return match_found;
    else
        last_match = match;

    puts(match);

    int new_profile = 0; // Default to Digital Profile
    size_t process_list_size = sizeof(process_list) / sizeof(process_list[0]);
    for (size_t i = 0; i < process_list_size; i++)
    {
        struct Process process = process_list[i];
        if (strcmp(match, process.match) == 0)
        {
            new_profile = process.profile;
            match_found = 1;
        }
    }

    if (last_profile != new_profile)
    {
        last_profile = new_profile;

        wooting_usb_send_feature(ActivateProfile, 0, 0, 0, new_profile);  // Change profile
        std_sleep(1);
        wooting_usb_send_feature(ReloadProfile0, 0, 0, 0, new_profile);   // Change RGB
        std_sleep(1);
        wooting_usb_send_feature(WootDevResetAll, 0, 0, 0, 0);            // Reset (Load RGB)
        std_sleep(1);
        wooting_usb_send_feature(RefreshRgbColors, 0, 0, 0, new_profile); // Refresh RGB (Load Effect)
    }

    return match_found;
}

void std_sleep(int seconds)
{
#ifdef _WIN32
        Sleep(seconds * 1000); // Fix keyboard spamming keypresses like Alt-Tab or Win when switching profiles
#else
        sleep(seconds);
#endif
}

void register_cleanup()
{
    atexit(cleanup);
    signal(SIGTERM, cleanup);
    signal(SIGINT, cleanup);
}