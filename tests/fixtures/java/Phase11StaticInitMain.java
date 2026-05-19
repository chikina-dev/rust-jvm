public class Phase11StaticInitMain {
    static int value;

    static {
        value = seed();
    }

    static int seed() {
        return 40;
    }

    public static int main() {
        return value + 2;
    }
}
