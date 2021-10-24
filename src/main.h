#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <signal.h>
#include <wooting-rgb-sdk.h>
#include <cJSON.h>

int main(void);
int update_profile(const char *name);
void std_sleep(int milliseconds);
void register_cleanup(void);
void load_config(void);
char* readFile(char* filename);