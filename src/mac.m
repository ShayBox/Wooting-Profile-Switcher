#include "mac.h"

@interface WootingProfileSwitcherListener : NSObject
- (void)windowLoop;
@end
@implementation WootingProfileSwitcherListener
- (void)windowLoop
{
    for (NSRunningApplication *currApp in [[NSWorkspace sharedWorkspace] runningApplications]) {
        if ([currApp isActive]) {
#ifdef _DEBUG
            NSLog(@"* %@", [currApp localizedName]);
#endif
            update_profile([[currApp localizedName] UTF8String]);
        }
    }
}
@end

@interface WootingWindowController : NSWindowController<NSWindowDelegate>
@end
@implementation WootingWindowController
- (BOOL)windowShouldClose:(id)sender {
    cleanup(0);
    return YES;
}
@end

void start_listening()
{
    [NSApplication sharedApplication];
    [NSBundle loadNibNamed:@"Wooting Profile Switcher" owner:NSApp];

    NSAutoreleasePool *p = [NSAutoreleasePool new];

    WootingProfileSwitcherListener *wpsl = [[WootingProfileSwitcherListener new] autorelease];
    WootingWindowController *wwc = [[WootingWindowController new] autorelease];

    NSUInteger windowStyle = NSTitledWindowMask | NSClosableWindowMask | NSResizableWindowMask;
    NSRect windowRect = NSMakeRect(100, 100, 400, 400);
    NSWindow * window = [[NSWindow alloc] initWithContentRect:windowRect
                                          styleMask:windowStyle
                                          backing:NSBackingStoreBuffered
                                          defer:NO];
    [window setTitle: @"Wooting Profile Switcher"];
    [window center];
    [window orderFrontRegardless];
    [window setDelegate:wwc];
    [window autorelease];

    NSWindowController * windowController = [[NSWindowController alloc] initWithWindow:window];
    [windowController autorelease];

    NSTextView * textView = [[NSTextView alloc] initWithFrame:windowRect];
    [textView autorelease];

    [window setContentView:textView];
    [textView insertText:@"Test"]; //TODO: Add usual console output to textview

    NSTimer *timer = [NSTimer scheduledTimerWithTimeInterval:1.0f
                                                      target:wpsl
                                                    selector:@selector(windowLoop)
                                                    userInfo:nil
                                                     repeats:YES];

    [NSApp activateIgnoringOtherApps:true];
    [NSApp run];
    [[NSRunLoop mainRunLoop] windowLoop];

    [p release];
}

const char *get_config_path()
{
    char *home = getenv("HOME");
    if (home)
    {
        const char *config_path = strcat(home, "/.config/WootingProfileSwitcher");

        struct stat sb;

        if (stat(config_path, &sb) != 0)
        {
            mkdir(strcat(home, "/.config"), S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
            mkdir(strcat(home, "/.config/WootingProfileSwitcher"), S_IRWXU | S_IRWXG | S_IROTH | S_IXOTH);
        }
        return strcat((char*)config_path, "/config.json");
    }

    return "./config.json";
}

void cleanup(int receivedSignal)
{
    reset_profile();
    wooting_rgb_reset();
    exit(EXIT_SUCCESS);
}
