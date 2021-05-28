CC?=gcc
CFLAGS?=-Os -s
LIBS?=-lX11 -lwooting-rgb-sdk

build: ../src/main.c ../src/linux.c
	$(CC) $(CFLAGS) $(LIBS) ../src/main.c ../src/linux.c -o WootingProfileSwitcher