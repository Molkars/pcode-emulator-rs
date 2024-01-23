
#include <stdio.h>

extern void exit(int);

int
main(int argc, char **argv)
{
    if (argc < 2) {
        fprintf(stderr, "usage: example [arg]\n");
        exit(1);
    }

    fprintf(stdout, "hi!\n");
    return 0;
}