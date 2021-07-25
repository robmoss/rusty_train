## n18token

- `n18token:Token`: it would be nice if each token owned its name, but then Token cannot implement Copy ...

  - This should be in the type and/or module documentation.

- `n18token:Tokens`: implement `IntoIterator` for `Item = (String, Token)` ... be we need to specify the exact `Iterator` type, so we'd have to make our own struct that implements `Iterator`.
