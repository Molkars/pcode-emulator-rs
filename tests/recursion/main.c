
int adder(int n) {
    if (n <= 0) return 0;
    return 1 + adder(n - 1);
}

int main() {
    int r = adder(4);
    return r;
}