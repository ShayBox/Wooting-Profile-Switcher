#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <signal.h>
#include <wooting-rgb-sdk.h>

int main(void);
int update_profile(const char *name);
void std_sleep(int milliseconds);
void register_cleanup(void);