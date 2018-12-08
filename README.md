This is an implementation in Rust of versioned treaps in a fully persistent,
purely functional style. It's largely based off a purely functional treap
[implementation in Haskell](https://wiki.haskell.org/The_Monad.Reader/Issue4/On_Treaps_And_Randomization),
although our interest is mainly in fast appends and clones.
