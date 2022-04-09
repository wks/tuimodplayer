# TUI Mod Player

TUIModPlayer is a [module music (a.k.a. mod)][mod] player with a [text-based
user interface (TUI)][tui-wiki] written in [Rust].

As the name suggests, it uses the [tui-rs] library to build the UI.

It uses [libopenmpt] to decode mods.  It can play whatever format libopenmpt
supports, including but not limited to `.mod`, `.xm`, `.s3m`, `.it`, `.mptm` and
so on.

[mod]: https://modarchive.org/index.php?article-modules
[tui-wiki]: https://en.wikipedia.org/wiki/Text-based_user_interface
[tui-rs]: https://crates.io/crates/tui
[libopenmpt]: https://lib.openmpt.org/libopenmpt/
[Rust]: https://www.rust-lang.org/

<!--
vim: tw=80
-->
