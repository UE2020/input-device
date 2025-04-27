#include <windows.h>
#include <winuser.h>
#include <stdio.h>

#pragma comment(lib, "user32.lib")

// Helper function to create a POINTER_TOUCH_INFO structure
POINTER_TOUCH_INFO createTouchInfo(POINTER_FLAGS pointerFlags, int x, int y, UINT32 pointerId) {
    POINTER_TOUCH_INFO contact;
    memset(&contact, 0, sizeof(POINTER_TOUCH_INFO));

    contact.pointerInfo.pointerType = PT_TOUCH;
    contact.pointerInfo.pointerId = pointerId;
    contact.pointerInfo.ptPixelLocation.x = x;
    contact.pointerInfo.ptPixelLocation.y = y;
    contact.pointerInfo.pointerFlags = pointerFlags;

    // Set contact area (optional, but good practice)
    // A small contact area around the touch point
    contact.rcContact.top = y - 2;
    contact.rcContact.bottom = y + 2;
    contact.rcContact.left = x - 2;
    contact.rcContact.right = x + 2;

    contact.touchFlags = TOUCH_FLAG_NONE;

    return contact;
}

int main() {
    // Initialize touch injection for a single contact
    // Requires Windows 8 or later
    if (!InitializeTouchInjection(10, TOUCH_FEEDBACK_DEFAULT)) {
        DWORD error = GetLastError();
        fprintf(stderr, "Failed to initialize touch injection. Error: %lu\n", error);
        if (error == ERROR_ACCESS_DENIED) {
            fprintf(stderr, "Touch injection requires administrative privileges or UI Access.\n");
        }
        return 1;
    }

    POINTER_TOUCH_INFO contact;
    
    // Simulate touch down
    printf("Simulating touch down at (300, 300)\n");
    contact = createTouchInfo(POINTER_FLAG_DOWN | POINTER_FLAG_INRANGE | POINTER_FLAG_INCONTACT, 300, 300, 0);
    if (!InjectTouchInput(1, &contact)) {
        fprintf(stderr, "Failed to inject touch down. Error: %lu\n", GetLastError());
        goto cleanup;
    }
    Sleep(50); // Short delay

    // Simulate touch move
    printf("Simulating touch move to (400, 350)\n");
    contact = createTouchInfo(POINTER_FLAG_UPDATE | POINTER_FLAG_INRANGE | POINTER_FLAG_INCONTACT, 400, 350, 0);
     if (!InjectTouchInput(1, &contact)) {
        fprintf(stderr, "Failed to inject touch move. Error: %lu\n", GetLastError());
        goto cleanup;
    }
    Sleep(50); // Short delay
    
    // Simulate another touch move
    printf("Simulating touch move to (500, 400)\n");
    contact = createTouchInfo(POINTER_FLAG_UPDATE | POINTER_FLAG_INRANGE | POINTER_FLAG_INCONTACT, 500, 400, 0);
     if (!InjectTouchInput(1, &contact)) {
        fprintf(stderr, "Failed to inject touch move. Error: %lu\n", GetLastError());
        goto cleanup;
    }
    Sleep(50); // Short delay

    // Simulate touch up
    printf("Simulating touch up at (500, 400)\n");
    contact = createTouchInfo(POINTER_FLAG_UP, 500, 400, 0);
    if (!InjectTouchInput(1, &contact)) {
        fprintf(stderr, "Failed to inject touch up. Error: %lu\n", GetLastError());
        goto cleanup;
    }

    printf("Touch simulation complete.\n");

cleanup:
    // There is no specific deinitialization function for touch injection.
    // The context is valid until the process terminates.

    return 0;
}