CC ?= gcc
CFLAGS ?= -Wall -pedantic -Os -s

SRCS = ../wooting-rgb-sdk/src/wooting-rgb-sdk.c ../wooting-rgb-sdk/src/wooting-usb.c ../cJSON/cJSON.c
LIBS = `pkg-config hidapi-hidraw --libs`
INCLUDES ?= `pkg-config hidapi-hidraw --cflags` -lX11 -I../wooting-rgb-sdk/src -I../cJSON

all: cjson wooting-rgb-sdk release

cjson:
	@echo "Building cjson"
	cd ../cJSON && make

wooting-rgb-sdk:
	@echo "Building wooting-rgb-sdk"
	cd ../wooting-rgb-sdk/linux && make

release: ../src/main.c ../src/linux.c
	$(CC) $(CFLAGS) $(LIBS) $(INCLUDES) $(SRCS) ../src/main.c ../src/linux.c -o WootingProfileSwitcher

debug: ../src/main.c ../src/linux.c
	CFLAGS+=-g3
	$(CC) $(CFLAGS) $(LIBS) $(INCLUDES) $(SRCS) ../src/main.c ../src/linux.c -o WootingProfileSwitcher