public class Phase8ReferenceArrayMain {
    int value;

    Phase8ReferenceArrayMain(int value) {
        this.value = value;
    }

    public static int main() {
        Phase8ReferenceArrayMain[] values = new Phase8ReferenceArrayMain[2];
        values[0] = null;
        values[1] = new Phase8ReferenceArrayMain(11);
        return values.length + values[1].value;
    }
}
