#define macros
APPNAME = WootingProfileSwitcher
EXECUTABLE_NAME_X86 = $(APPNAME)_x86.exe
EXECUTABLE_NAME = $(APPNAME).exe
DIR_SRC = ..\src
DIR_INCLUDE = .\include\x64\wooting-rgb-sdk
DIR_INCLUDE_X86 = .\include\x86\wooting-rgb-sdk
DIR_LIB = .\lib\x64
DIR_LIB_X86 = .\lib\x86
LIB_DEPENDENCIES = ucrt.lib User32.lib wooting-rgb-sdk64.lib
LIB_DEPENDENCIES_X86 = ucrt.lib User32.lib wooting-rgb-sdk.lib

# Flexible folder macros
DIR_BUILD = .\build
DIR_BIN = $(DIR_BUILD)\bin
DIR_REL_BASE = $(DIR_BIN)\release
DIR_REL = $(DIR_REL_BASE)\x64
DIR_REL_X86 = $(DIR_REL_BASE)\x86
DIR_DBG_BASE = $(DIR_BIN)\debug
DIR_DBG = $(DIR_DBG_BASE)\x64
DIR_DBG_X86 = $(DIR_DBG_BASE)\x86
DIR_INTERMEDIATE_BASE = $(DIR_BUILD)\intermediate
DIR_INTERMEDIATE_REL_BASE = $(DIR_INTERMEDIATE_BASE)\release
DIR_INTERMEDIATE_REL = $(DIR_INTERMEDIATE_REL_BASE)\x64
DIR_INTERMEDIATE_REL_X86 = $(DIR_INTERMEDIATE_REL_BASE)\x86
DIR_INTERMEDIATE_DBG_BASE = $(DIR_INTERMEDIATE_BASE)\debug
DIR_INTERMEDIATE_DBG = $(DIR_INTERMEDIATE_DBG_BASE)\x64
DIR_INTERMEDIATE_DBG_X86 = $(DIR_INTERMEDIATE_DBG_BASE)\x86

# Linker arguments
LINK_LEAN = /D "WIN32_LEAN_AND_MEAN" /D "VC_EXTRALEAN"
LINK_ARGS = /NODEFAULTLIB /ENTRY:main

# Linker macro for use in compiler macros
LINK_X86 = /link /MACHINE:X86 $(LINK_ARGS) /LIBPATH:$(DIR_LIB_X86) $(LIB_DEPENDENCIES_X86)
LINK_X64 = /link /MACHINE:X64 $(LINK_ARGS) /LIBPATH:$(DIR_LIB) $(LIB_DEPENDENCIES)

# compiler macro
COMPILER_X86 = \$(EXECUTABLE_NAME_X86) /EHsc /D "_X86_" $(LINK_LEAN) /I$(DIR_INCLUDE_X86) $(SRC_FILES) $(LINK_X86)
COMPILER_X64 = \$(EXECUTABLE_NAME) /EHsc /D "_X64_" $(LINK_LEAN) /I$(DIR_INCLUDE) $(SRC_FILES) $(LINK_X64)

# Debug macro
DEBUG = /D "DEBUG"

# Macro for all used source files
SRC_FILES= \
  $(DIR_SRC)\main.c \
  $(DIR_SRC)\windows.c

# description block
$(EXECUTABLE_NAME_X86): $(SRC_FILES)
  cl /Fe$(DIR_REL_X86)$(COMPILER_X86)
  copy *.obj $(DIR_INTERMEDIATE_REL_X86)
  del *.obj
  copy $(DIR_LIB_X86)\wooting-rgb-sdk.dll $(DIR_REL_X86)

$(EXECUTABLE_NAME): $(SRC_FILES)
  cl /Fe$(DIR_REL)$(COMPILER_X64)
  copy *.obj $(DIR_INTERMEDIATE_REL)
  del *.obj
  copy $(DIR_LIB)\wooting-rgb-sdk.dll $(DIR_REL)

$(EXECUTABLE_NAME_X86)_DBG: $(SRC_FILES)
  cl $(DEBUG) /Fe$(DIR_DBG_X86)$(COMPILER_X86) /VERBOSE
  copy *.obj $(DIR_INTERMEDIATE_DBG_X86)
  del *.obj
  copy $(DIR_LIB_X86)\wooting-rgb-sdk.dll $(DIR_DBG_X86)

$(EXECUTABLE_NAME)_DBG: $(SRC_FILES)
  cl $(DEBUG) /Fe$(DIR_DBG)$(COMPILER_X64) /VERBOSE
  copy *.obj $(DIR_INTERMEDIATE_DBG)
  del *.obj
  copy $(DIR_LIB)\wooting-rgb-sdk.dll $(DIR_DBG)

# build application
wooting_profile_switcher: $(EXECUTABLE_NAME_X86) $(EXECUTABLE_NAME)
wooting_profile_switcher_debug: $(EXECUTABLE_NAME_X86)_DBG $(EXECUTABLE_NAME)_DBG

# create output directories
create_dirs_base:
 @if not exist $(DIR_BUILD) mkdir $(DIR_BUILD)
 @if not exist $(DIR_BIN) mkdir $(DIR_BIN)
 @if not exist $(DIR_REL_BASE) mkdir $(DIR_REL_BASE)
 @if not exist $(DIR_DBG_BASE) mkdir $(DIR_DBG_BASE)
 @if not exist $(DIR_INTERMEDIATE_BASE) mkdir $(DIR_INTERMEDIATE_BASE)
 @if not exist $(DIR_INTERMEDIATE_REL_BASE) mkdir $(DIR_INTERMEDIATE_REL_BASE)
 @if not exist $(DIR_INTERMEDIATE_DBG_BASE) mkdir $(DIR_INTERMEDIATE_DBG_BASE)

create_dirs:
 @if not exist $(DIR_REL) mkdir $(DIR_REL)
 @if not exist $(DIR_REL_X86) mkdir $(DIR_REL_X86)
 @if not exist $(DIR_INTERMEDIATE_REL_X86) mkdir $(DIR_INTERMEDIATE_X86)
 @if not exist $(DIR_INTERMEDIATE_REL) mkdir $(DIR_INTERMEDIATE_REL)

create_dirs_dbg:
 @if not exist $(DIR_DBG) mkdir $(DIR_DBG)
 @if not exist $(DIR_DBG_X86) mkdir $(DIR_DBG_X86)
 @if not exist $(DIR_INTERMEDIATE_DBG_X86) mkdir $(DIR_INTERMEDIATE_DBG_X86)
 @if not exist $(DIR_INTERMEDIATE_DBG) mkdir $(DIR_INTERMEDIATE_DBG)

# delete output directories
clean:
 @if exist $(DIR_REL_BASE) rmdir /S /Q $(DIR_REL_BASE)
 @if exist $(DIR_INTERMEDIATE_REL_BASE) rmdir /S /Q $(DIR_INTERMEDIATE_REL_BASE)

clean_dbg:
 @if exist $(DIR_DBG_BASE) rmdir /S /Q $(DIR_DBG_BASE)
 @if exist $(DIR_INTERMEDIATE_DBG_BASE) rmdir /S /Q $(DIR_INTERMEDIATE_DBG_BASE)

# create directories and build application

all: clean create_dirs_base create_dirs wooting_profile_switcher

debug: clean_dbg create_dirs_base create_dirs_dbg wooting_profile_switcher_debug