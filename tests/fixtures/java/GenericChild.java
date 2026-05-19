interface Named {
  default String label() {
    return "named";
  }
}

abstract class GenericBase<T extends Number> {
  abstract T value();
}

public class GenericChild extends GenericBase<Integer> implements Named {
  public Integer value() {
    return 42;
  }

  public String label() {
    return Named.super.label() + value();
  }
}
