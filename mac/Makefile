CC ?= gcc
CFLAGS ?= -ObjC -Wall -pedantic -Os

SRCS = ../wooting-rgb-sdk/src/wooting-rgb-sdk.c ../wooting-rgb-sdk/src/wooting-usb.c ../cJSON/cJSON.c
LIBS = `pkg-config hidapi --libs`
INCLUDES ?= `pkg-config hidapi --cflags` -I../wooting-rgb-sdk/src -I../cJSON
LDLIBS = -framework ApplicationServices

all: cjson wooting-rgb-sdk release

cjson:
	@echo "Building cjson"
	cd ../cJSON && make

wooting-rgb-sdk:
	@echo "Building wooting-rgb-sdk"
	cd ../wooting-rgb-sdk/mac && make

release: ../src/main.c ../src/macos.c
	$(CC) $(CFLAGS) $(LIBS) $(LDLIBS) $(INCLUDES) $(SRCS) ../src/main.c ../src/macos.c -o WootingProfileSwitcher
	cp ../example_config.json config.json

debug: ../src/main.c ../src/macos.c
	CFLAGS+=-g3
	$(CC) $(CFLAGS) $(LIBS) $(LDLIBS) $(INCLUDES) $(SRCS) ../src/main.c ../src/macos.c -o WootingProfileSwitcher
	cp ../example_config.json config.json