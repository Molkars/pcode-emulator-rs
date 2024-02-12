
int ident(int n) {
    int sum = 0;
    for (int i = 1; i < n; i++) {
        sum += i;
    }
    return sum;
}

int main() {
    int a = ident(2);
    return a;
}
