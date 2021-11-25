#include "macos.h"

void start_listening()
{
    while (true)
    {
        OSErr err;
        ProcessSerialNumber psn;
        ProcessInfoRec info;
        StringPtr process_name = malloc(64);

        err = GetFrontProcess(&psn);
        if (err == noErr)
        {
            bzero(process_name, 64);
            if (process_name)
            {
                info.processInfoLength = sizeof(ProcessInfoRec);
                info.processName = process_name;
                err = GetProcessInformation(&psn, &info);

                if (err == noErr)
                {
                    update_profile(process_name+1);
                }
            }
        }

        free(&err);
        free(process_name);
    }
}

const char *get_config_path()
{
    char *home = getenv("HOME");
    if (home)
    {
        
        const char config_location = strcat(home, "/.config/WootingProfileSwitcher/config.json");
        
        if (access(config_location, F_OK) == 0)
        {
            return config_location;
        }
    }

    return "./config.json";
}

void cleanup(void)
{
    reset_profile();
    wooting_rgb_reset();
    exit(EXIT_SUCCESS);
}
