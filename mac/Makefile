CC ?= gcc
CFLAGS ?= -ObjC -Wall -pedantic -Os

SRCS = ../wooting-rgb-sdk/src/wooting-rgb-sdk.c ../wooting-rgb-sdk/src/wooting-usb.c ../cJSON/cJSON.c
LIBS = `pkg-config hidapi --libs`
INCLUDES ?= `pkg-config hidapi --cflags` -I../wooting-rgb-sdk/src -I../cJSON
LDLIBS = -framework Carbon -framework AppKit -framework Cocoa

BUNDLENAME = Wooting\ Profile\ Switcher.app
APPNAME = WootingProfileSwitcher

all: cjson wooting-rgb-sdk release create_bundle

cjson:
	@echo "Building cjson"
	cd ../cJSON && make

wooting-rgb-sdk:
	@echo "Building wooting-rgb-sdk"
	cd ../wooting-rgb-sdk/mac && make

release: ../src/main.c ../src/mac.m
	rm -rf $(BUNDLENAME)
	$(CC) $(CFLAGS) $(LIBS) $(LDLIBS) $(INCLUDES) $(SRCS) ../src/main.c ../src/mac.m -o $(APPNAME)

debug: ../src/main.c ../src/mac.m
	CFLAGS+=-g3
	rm -rf $(BUNDLENAME)
	$(CC) $(CFLAGS) $(LIBS) $(LDLIBS) $(INCLUDES) $(SRCS) ../src/main.c ../src/mac.m -o $(APPNAME)

create_bundle:
	mkdir $(BUNDLENAME)
	cp Info.plist $(BUNDLENAME)/Info.plist
	cp $(APPNAME).icns $(BUNDLENAME)/$(APPNAME).icns
	cp $(APPNAME) $(BUNDLENAME)/$(APPNAME)