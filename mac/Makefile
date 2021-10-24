CC?=gcc
CFLAGS?=-Wall -pedantic -Os -s
LIBS?=-lX11 -lwooting-rgb-sdk

release: ../src/main.c ../src/mac.c ../cJSON/cJSON.c
	$(CC) $(CFLAGS) $(LIBS) ../src/main.c ../src/mac.c -o WootingProfileSwitcher

debug: ../src/main.c ../src/mac.c ../cJSON/cJSON.c
	CFLAGS+=-g3
	$(CC) $(CFLAGS) $(LIBS) ../src/main.c ../src/mac.c -o WootingProfileSwitcher