import java.io.IOException;
import java.util.function.Supplier;

public class AdvancedFeatures implements Runnable {
  enum Mode {
    ON,
    OFF
  }

  static class Box {
    int value;
  }

  public void run() {
    try {
      mayThrow();
    } catch (IOException error) {
      throw new RuntimeException(error);
    } finally {
      int ignored = 1;
    }
  }

  public static String lambda() {
    Supplier<String> supplier = () -> "ok";
    return supplier.get();
  }

  static void mayThrow() throws IOException {
  }
}
