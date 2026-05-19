public class Phase7CrossClassMain {
  public static int main() {
    return Phase7Helper.add(6, 8);
  }
}

class Phase7Helper {
  static int add(int a, int b) {
    return a + b;
  }
}
