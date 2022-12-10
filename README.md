# pw

Generates random passwords with some more integration into clipboard and history.

# USAGE

To create a new password:

```
pw [-m <message>]
```

This will generate some passwords and open a curses interface for you to choose one.
The one you pick will be copied to the clipboard so you can paste it somewhere.
We'll also keep a history of your choices in `~/.pw/history.toml`.
Each history entry has the password, a timestamp, and optionally the *message* from the `-m` option.
