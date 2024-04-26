#include <stdio.h>
#include <stdint.h>
#include <mic2.h>
#include <cstring>

void test_error_strings() {
    char buffer[255];
    uint32_t length = 255;
    int result = mic2_error_string(NeoVIMICErrTypeSuccess, buffer, &length);
    if (result != NeoVIMICErrTypeSuccess) {
        printf("mic2_error_string() error: %d\n", result);
        return;
    }
    printf("%s\n", buffer);

    result = mic2_error_string(NeoVIMICErrTypeFailure, buffer, &length);
    if (result != NeoVIMICErrTypeSuccess) {
        printf("mic2_error_string() error: %d\n", result);
        return;
    }
    printf("%s\n", buffer);

    result = mic2_error_string(NeoVIMICErrTypeInvalidParameter, buffer, &length);
    if (result != NeoVIMICErrTypeSuccess) {
        printf("mic2_error_string() error: %d\n", result);
        return;
    }
    printf("%s\n", buffer);
}

int main(int argc, char* argv[]) {
    (void)argc; (void)argv;

    NeoVIMIC* devices = NULL;
    uint32_t count = 0;
    printf("Finding devices...\n");
    int result = mic2_find(&devices, &count);
    if (result != NeoVIMICErrTypeSuccess) {
        char buffer[255];
        uint32_t length = 255;
        int result2 = mic2_error_string(result, buffer, &length);
        if (result2 != NeoVIMICErrTypeSuccess) {
            printf("mic2_error_string() error: %d\n", result2);
            return 1;
        }
        printf("mic2_find() error: %s (%d)\n", buffer, result);
        return 1;
    }
    printf("Found %d devices\n", count);

    test_error_strings();

    return 0;
}