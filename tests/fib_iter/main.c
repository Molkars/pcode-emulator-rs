
int fib(int n) {
    long long a = 0, b = 1, c;
    for (int i = 0; i < n; i++) {
        c = a + b;
        a = b;
        b = c;
    }
    return a;
}

int main(int argc, char **argv) {
    int r = fib(10);
    return r;
}

