public class RuntimeShapes {
  public static long mix(int seed) {
    long value = seed;
    value = (value << 3) ^ 0xCAFE_BABEL;
    double d = value / 3.0d;
    return value + (long) d;
  }

  public static int arrays(Object input) {
    int[][] grid = new int[2][3];
    grid[1][2] = input instanceof String ? ((String) input).length() : 5;
    Object[] refs = new String[] {"a", "bb"};
    return grid[1][2] + refs.length;
  }
}
