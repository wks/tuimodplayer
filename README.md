# TUI Mod Player

*Q: What would I do if I [downloaded][ModArchive-BT] the entire [ModArchive] and
don't know what to do with the 100000+ mod files I have?*

*A: You play them with [TUIModPlayer].*

TUIModPlayer is a [module music (a.k.a. mod)][mod] player with a [text-based
user interface (TUI)][tui-wiki] written in [Rust].

It uses [ratatui] for its user interface.  It used to use [tui-rs], and that's
is why it was called *TUI* Mod Player in the first place.

It uses [libopenmpt] to decode mods.  It can play whatever format libopenmpt
supports, including but not limited to `.mod`, `.xm`, `.s3m`, `.it`, `.mptm` and
so on.

[libopenmpt]: https://lib.openmpt.org/libopenmpt/
[ModArchive-BT]: http://tracker.modarchive.org/
[ModArchive]: https://modarchive.org/
[mod]: https://modarchive.org/index.php?article-modules
[Rust]: https://www.rust-lang.org/
[TUIModPlayer]: https://github.com/wks/tuimodplayer
[tui-rs]: https://crates.io/crates/tui
[tui-wiki]: https://en.wikipedia.org/wiki/Text-based_user_interface
[ratatui]: https://ratatui.rs/

# How to Use?

## Download, Compile and Run

You may need to install libopenmpt using your favourite package managers, for
example:

```sh
sudo pacman -S libopenmpt
```

Then clone this repo and build/run.

```sh
git clone https://github.com/wks/tuimodplayer.git
cd tuimodplayer
cargo run --release -- /path/to/modarchive_2007_official_snapshot_120000_modules
```

If you would like to randomise the playlist, add the `-s` option.

```sh
cargo run --release -- /path/to/modarchive_2007_official_snapshot_120000_modules -s
```

## Key Bindings

List available key bindings:

```sh
openmpt123 --help-keyboard
```

Yes.  This is intentional.  I don't want to surprise those who are used to
openmpt123.  But there are more:

-   `r`: Toggle repeating.  When on, it will play the same mod repeatedly
    forever.
-   `ctrl+L`: Redraw screen.

# Author

Kunshan Wang \<d2tzMTk4NkBnbWFpbC5jb20K\>

# License

GPLv3

<!--
vim: tw=80
-->
