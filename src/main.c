#ifdef _WIN32
#include "win_native.h"
#elif __APPLE__
#include "mac.h"
#elif __linux__
#include "linux.h"
#endif
#include "main.h"

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

    // process_list = // TODO: Add config

    start_listening();
    wooting_rgb_reset();
    return EXIT_SUCCESS;
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

        // https://gist.github.com/BigBrainAFK/0ba454a1efb43f7cb6301cda8838f432
        const char ActivateProfile = 23;
        const char ReloadProfile0 = 7;
        const char WootDevResetAll = 32;
        const char RefreshRgbColors = 29;
        wooting_usb_send_feature(ActivateProfile, 0, 0, 0, new_profile);  // Change profile
        wooting_usb_send_feature(ReloadProfile0, 0, 0, 0, new_profile);   // Change RGB
        wooting_usb_send_feature(WootDevResetAll, 0, 0, 0, 0);            // Reset (Load RGB)
        wooting_usb_send_feature(RefreshRgbColors, 0, 0, 0, new_profile); // Refresh RGB (Load Effect)
    }
    return match_found;
}