#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <signal.h>
#include <wooting-rgb-sdk.h>
#include <cJSON.h>

int main(void);
void reset_profile(void);
int update_profile(const char *name);
void set_profile(int profileIndex);
void register_cleanup(void);
void load_config(void);
char* read_file(char* filename);
