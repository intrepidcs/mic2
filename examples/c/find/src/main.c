#include <stdio.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
#else
#include <unistd.h>
#endif

#include <mic2.h>

#define DEVICE_COUNT (10)

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
    NeoVIMIC devices[DEVICE_COUNT] = {0};
    //memset(devices, 0, sizeof(devices)*DEVICE_COUNT);
    uint32_t length = (uint32_t)DEVICE_COUNT;
    NeoVIMICErrType err = NeoVIMICErrTypeFailure;
    if ((err = mic2_find(devices, &length, MIC2_API_VERSION, sizeof(NeoVIMIC))) != NeoVIMICErrTypeSuccess) {
        return print_error(&err);
    }
    printf("Found %d neoVI MIC2 devices!\n", length);

    for (uint32_t i = 0; i < length; i++) {
        printf("Opening IO device %s...\n", devices[i].serial_number);
        if ((err = mic2_io_open(&devices[i])) != NeoVIMICErrTypeSuccess) {
            return print_error(&err);
        }

        bool is_enabled = false;
        if ((err = mic2_io_buzzer_is_enabled(&devices[i], &is_enabled)) != NeoVIMICErrTypeSuccess) {
            return print_error(&err);
        }
        printf("IO device Buzzer is enabled? %d...\n", is_enabled);

        if ((err = mic2_io_buzzer_enable(&devices[i], true)) != NeoVIMICErrTypeSuccess) {
            return print_error(&err);
        }
        sleep(1);
        if ((err = mic2_io_buzzer_enable(&devices[i], false)) != NeoVIMICErrTypeSuccess) {
            return print_error(&err);
        }

        printf("Closing IO device %s...\n", devices[i].serial_number);
        if ((err = mic2_io_close(&devices[i])) != NeoVIMICErrTypeSuccess) {
            return print_error(&err);
        }
        mic2_free(&devices[i]);
    }
    return 0;
}
