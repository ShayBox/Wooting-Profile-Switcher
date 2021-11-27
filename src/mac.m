#include "mac.h"

NSTextView *textView;
NSAutoreleasePool *p;

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
    return true;
}
@end

WootingProfileSwitcherListener *wpsl;

int init_window()
{
    [NSApplication sharedApplication];
    [NSBundle loadNibNamed:@"Wooting Profile Switcher" owner:NSApp];

    p = [NSAutoreleasePool new];

    wpsl = [[WootingProfileSwitcherListener new] autorelease];
    WootingWindowController *wwc = [[WootingWindowController new] autorelease];

    NSUInteger windowStyle = NSTitledWindowMask | NSClosableWindowMask | NSWindowStyleMaskMiniaturizable;
    NSRect windowRect = NSMakeRect(100, 100, 400, 400);
    NSWindow *window = [[NSWindow alloc] initWithContentRect:windowRect
                                          styleMask:windowStyle
                                          backing:NSBackingStoreBuffered
                                          defer:NO];
    [window setTitle: @"Wooting Profile Switcher"];
    [window setBackgroundColor: NSColor.blackColor];
    [window center];
    [window orderFrontRegardless];
    [window setDelegate:wwc];
    [window autorelease];

    NSWindowController *windowController = [[NSWindowController alloc] initWithWindow:window];
    [windowController autorelease];

    textView = [[NSTextView alloc] initWithFrame:windowRect];
    [textView setEditable:false];
    [textView autorelease];

    [window setContentView:textView];

    append_text_to_view("Wooting Profile Switcher started!\n");
    //[textView insertText:@"Test"]; //TODO: Add usual console output to textview

    return 1;
}

void start_listening()
{
    NSTimer *timer = [NSTimer scheduledTimerWithTimeInterval:1.0f
                                                        target:wpsl
                                                        selector:@selector(windowLoop)
                                                        userInfo:nil
                                                        repeats:true];

    [NSApp activateIgnoringOtherApps:true];
    [NSApp run];
    [[NSRunLoop mainRunLoop] windowLoop];

    [p release];
}

void append_text_to_view(char* text)
{
    NSDictionary *attrs = @{ NSForegroundColorAttributeName : NSColor.whiteColor };
    NSString *string = [NSString stringWithUTF8String: text];
    NSAttributedString* attr = [[NSAttributedString alloc] initWithString:string attributes:attrs];

    [[textView textStorage] appendAttributedString:attr];
    [textView scrollRangeToVisible:NSMakeRange([[textView string] length], 0)];
}

void append_error_to_view(char* text)
{
    NSDictionary *attrs = @{ NSForegroundColorAttributeName : NSColor.redColor };
    NSString *string = [NSString stringWithUTF8String: text];
    NSAttributedString* attr = [[NSAttributedString alloc] initWithString:string attributes:attrs];

    [[textView textStorage] appendAttributedString:attr];
    [textView scrollRangeToVisible:NSMakeRange([[textView string] length], 0)];
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
