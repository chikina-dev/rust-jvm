public class Phase8ReferenceArrayMain {
    int value;

    Phase8ReferenceArrayMain(int value) {
        this.value = value;
    }

    public static int main() {
        Phase8ReferenceArrayMain[] values = new Phase8ReferenceArrayMain[1];
        values[0] = new Phase8ReferenceArrayMain(11);
        return values[0].value;
    }
}
