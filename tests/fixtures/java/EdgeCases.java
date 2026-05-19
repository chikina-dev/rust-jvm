import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

@Retention(RetentionPolicy.RUNTIME)
@Target({ElementType.TYPE, ElementType.TYPE_USE, ElementType.TYPE_PARAMETER, ElementType.METHOD, ElementType.PARAMETER})
@interface Marker {
  int value();
  Class<?> type();
  String name() default "ok";
}

@Marker(value = 7, type = String.class)
public class EdgeCases<@Marker(value = 1, type = Object.class) T> {
  static final long L = 1234567890123L;
  static final double D = 1.25d;

  public static int denseSwitch(int x) {
    switch (x) {
      case 0:
        return 10;
      case 1:
        return 11;
      case 2:
        return 12;
      default:
        return -1;
    }
  }

  public static int sparseSwitch(int x) {
    switch (x) {
      case 1:
        return 10;
      case 100:
        return 20;
      case 1000:
        return 30;
      default:
        return -1;
    }
  }

  public @Marker(value = 2, type = String.class) String typeUse(@Marker(value = 3, type = String.class) String input) {
    return input;
  }
}
