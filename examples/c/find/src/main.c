#include <stdio.h>
#include <string.h>

#include <mic2.h>

int print_error(const NeoVIMICErrType* err) {
    // Check for invalid parameter
    if (!err) {
        printf("print_error(): Invalid parameter\n");
        return NeoVIMICErrTypeInvalidParameter;
    }

    // Get error string and print it
    const size_t BUF_SIZE = 1024;
    char buffer[BUF_SIZE];
    memset(buffer, 0, BUF_SIZE);
    uint32_t length = (uint32_t)BUF_SIZE;
    if (mic2_error_string(*err, buffer, &length) == NeoVIMICErrTypeSuccess) {
        printf("%s\n", buffer);
    } else {
        printf("Failed to get error string: %d\n", *err);
    }
    return (int)*err;
}

int main(int argc, char* argv[]) {
    (void)argc;
    (void)argv;

    printf("Finding neovi MIC2 devices...\n");
    NeoVIMIC* devices = NULL;
    uint32_t length = 0;
    NeoVIMICErrType err = NeoVIMICErrTypeFailure;
    if ((err = mic2_find(devices, &length)) != NeoVIMICErrTypeSuccess) {
        return print_error(&err);
    }
    printf("Found %d neoVI MIC2 devices!\n", length);

    for (uint32_t i = 0; i < length; i++) {

        mic2_free(devices[i]);
    }
    return 0;
}
