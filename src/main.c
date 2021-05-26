#ifdef _WIN32
#include "win_native.h"
#elif __APPLE__
#include "mac.h"
#elif __linux__
#include "linux.h"
#endif
#include "main.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <wooting-rgb-sdk.h>
#include "windows.h"

struct Process
{
    const char *name;
    const int profile;
};

// TODO: Add config file
const struct Process process_list[] = {
#ifdef _WIN32
    {"", 0},
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

    start_listening();
    wooting_rgb_reset();
    return EXIT_SUCCESS;
}

int last_profile = -1;
void update_profile(const char *name)
{
    puts(name); // Process name for users

    int new_profile = 0; // Default to Digital Profile
    size_t process_list_size = sizeof(process_list) / sizeof(process_list[0]);
    for (size_t i = 0; i < process_list_size; i++)
    {
        struct Process process = process_list[i];
        if (strcmp(name, process.name) == 0)
            new_profile = process.profile;
    }

    if (last_profile != new_profile)
    {
        last_profile = new_profile;

        // https://gist.github.com/BigBrainAFK/0ba454a1efb43f7cb6301cda8838f432
        const char ActivateProfile = 23;
        const char ReloadProfile0 = 7;
        const char WootDevResetAll = 32;
        const char RefreshRgbColors = 29;
        // wooting_usb_send_feature(ActivateProfile, 0, 0, 0, new_profile);  // Change profile
        // wooting_usb_send_feature(ReloadProfile0, 0, 0, 0, new_profile);   // Change RGB
        // wooting_usb_send_feature(WootDevResetAll, 0, 0, 0, 0);            // Reset (Load RGB)
        // wooting_usb_send_feature(RefreshRgbColors, 0, 0, 0, new_profile); // Refresh RGB (Load Effect)
    }
}