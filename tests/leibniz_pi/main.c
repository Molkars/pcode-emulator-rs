// for those who are unaware
// (pi/4) = sum(k=0, k<inf, (-1)^k/(2k+1))

// compile with -lm
double pow(double, double);

double leibniz(double n) {
    double result = 0;
    for (int i = 0; i < n; i++) {
        double term = pow((double) -1.0, (double) i) / (double) (2 * i + 1);
    }
    return 4.0 * result;
}

int main() {
    double pi = leibniz(15);
    int out = pi * 1000000;
    return out;
}