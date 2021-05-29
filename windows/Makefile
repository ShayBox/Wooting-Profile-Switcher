#define macros
APPNAME = WootingProfileSwitcher
EXECUTABLE_NAME_X86 = $(APPNAME)_x86.exe
EXECUTABLE_NAME = $(APPNAME).exe

BIN32 = %VCToolsInstallDir%bin\Hostx86\x86
BIN64 = %VCToolsInstallDir%bin\Hostx64\x64
AUX_DIR = %VCToolsInstallDir%..\..\..\Auxiliary\Build

DIR_SRC = ..\src
DIR_INCLUDE = .\include\wooting-rgb-sdk
DIR_LIB = .\lib\x64
DIR_LIB_X86 = .\lib\x86
LIB_DEPENDENCIES = Kernel32.lib ucrt.lib User32.lib libcmt.lib wooting-rgb-sdk64.lib
LIB_DEPENDENCIES_X86 = Kernel32.lib ucrt.lib User32.lib libcmt.lib wooting-rgb-sdk.lib

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
LINK_ARGS = /NOLOGO /NODEFAULTLIB /SUBSYSTEM:CONSOLE /ENTRY:main %LINK_DBG%

# Linker macro for use in compiler macros
LINK_X86 = /link /MACHINE:X86 $(LINK_ARGS) /LIBPATH:$(DIR_LIB_X86) $(LIB_DEPENDENCIES_X86)
LINK_X64 = /link /MACHINE:X64 $(LINK_ARGS) /LIBPATH:$(DIR_LIB) $(LIB_DEPENDENCIES)

#DEBUG macro
DEBUG = %DEBUG_FLAG%

# compiler macro
COMPILER_X86 = "$(BIN32)\cl.exe" $(DEBUG) /utf-8 /arch:IA32 /Fe%DIR_COMP_X86%\$(EXECUTABLE_NAME_X86) /EHsc /DWIN32 /D "_X86_" $(LINK_LEAN) /I$(DIR_INCLUDE) $(SRC_FILES) $(LINK_X86)
COMPILER_X64 = "$(BIN64)\cl.exe" $(DEBUG) /utf-8 /Fe%DIR_COMP%\$(EXECUTABLE_NAME) /EHsc /DWIN64 /D "_X64_" $(LINK_LEAN) /I$(DIR_INCLUDE) $(SRC_FILES) $(LINK_X64)

# Macro for all used source files
SRC_FILES= \
  $(DIR_SRC)\main.c \
  $(DIR_SRC)\win_native.c

# description block
$(EXECUTABLE_NAME_X86): $(SRC_FILES)
  $(COMPILER_X86)
  @copy *.obj "%DIR_INTERMEDIATE_COMP_X86%""
  @del *.obj
  @copy $(DIR_LIB_X86)\wooting-rgb-sdk.dll "%DIR_COMP_X86%"

$(EXECUTABLE_NAME): $(SRC_FILES)
  $(COMPILER_X64)
  @copy *.obj "%DIR_INTERMEDIATE_COMP%"
  @del *.obj
  @copy $(DIR_LIB)\wooting-rgb-sdk64.dll "%DIR_COMP%"

# build application
wooting_profile_switcher_debug: set_debug wooting_profile_switcher

set_debug:
  @set DEBUG_BUILD=TRUE
  @set DIR_COMP_X86=$(DIR_DBG_X86)
  @set DIR_COMP=$(DIR_DBG)
  @set DIR_INTERMEDIATE_COMP_X86=$(DIR_INTERMEDIATE_DBG_X86)
  @set DIR_INTERMEDIATE_COMP=$(DIR_INTERMEDIATE_DBG)
  @set LINK_DBG=^/VERBOSE
  @set DEBUG_FLAG=^/MDd

set_env:
  @set DIR_COMP_X86=$(DIR_REL_X86)
  @set DIR_COMP=$(DIR_REL)
  @set DIR_INTERMEDIATE_COMP_X86=$(DIR_INTERMEDIATE_REL_X86)
  @set DIR_INTERMEDIATE_COMP=$(DIR_INTERMEDIATE_REL)
  @set LINK_DBG=""
  @set DEBUG_FLAG=""

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
 @if not exist $(DIR_INTERMEDIATE_REL) mkdir $(DIR_INTERMEDIATE_REL)
 @if not exist $(DIR_INTERMEDIATE_REL_X86) mkdir $(DIR_INTERMEDIATE_REL_X86)

create_dirs_dbg:
 @if not exist $(DIR_DBG) mkdir $(DIR_DBG)
 @if not exist $(DIR_DBG_X86) mkdir $(DIR_DBG_X86)
 @if not exist $(DIR_INTERMEDIATE_DBG) mkdir $(DIR_INTERMEDIATE_DBG)
 @if not exist $(DIR_INTERMEDIATE_DBG_X86) mkdir $(DIR_INTERMEDIATE_DBG_X86)

# delete output directories
clean:
 @if exist $(DIR_REL_BASE) rmdir /S /Q $(DIR_REL_BASE)
 @if exist $(DIR_INTERMEDIATE_REL_BASE) rmdir /S /Q $(DIR_INTERMEDIATE_REL_BASE)

clean_dbg:
 @if exist $(DIR_DBG_BASE) rmdir /S /Q $(DIR_DBG_BASE)
 @if exist $(DIR_INTERMEDIATE_DBG_BASE) rmdir /S /Q $(DIR_INTERMEDIATE_DBG_BASE)

# create directories and build application

release32: clean create_dirs_base create_dirs set_env $(EXECUTABLE_NAME_X86)
release64: clean create_dirs_base create_dirs set_env $(EXECUTABLE_NAME)

debug32: clean_dbg create_dirs_base create_dirs_dbg set_debug $(EXECUTABLE_NAME_X86)
debug64: clean_dbg create_dirs_base create_dirs_dbg set_debug $(EXECUTABLE_NAME)