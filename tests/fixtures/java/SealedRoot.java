public sealed interface SealedRoot permits SealedLeaf {
}

final class SealedLeaf implements SealedRoot {
}
