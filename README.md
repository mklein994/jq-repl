# jq-repl

An interactive JSON explorer.

This is essentially a mix of shell tools that are useful for exploring JSON documents put glued together to produce an interactive tool for transforming, viewing, and otherwise exploring JSON.

> **Warning**
>
> :warning: This project is highly experimental, and until the 1.0 release, expect breaking changes that don't follow Semantic Versioning yet. Plans to support SemVer will be considered once this feels Stable™[^1].

## Usage

```console
jq-repl ./foo.json
jq-repl <(jo -fruit=$(jo -a apple banana cherry))
cargo metadata --format-version=1 | jq-repl
jo foo=bar | jq-repl - ./baz.json
```

Double check you have the necessary binaries installed:

```console
$ jq-repl --version-verbose
jq-repl x.y.z

fzf:	a.b (foobar)
gojq:	...
# ...
```

If you're ever are curious what it does behind-the-scenes, pass `--show-fzf-command` to give a rough Bash shell script of what it's actually constructing. Pass arguments to see how it changes.

This tool was built using the amazing work done by the tools it calls. Note that the defaults are highly opinionated, as this was built to scratch my own itch. There are flags you can pass to override the defaults. Here is a breakdown of some of the key programs called:

- [`fzf`](https://github.com/junegunn/fzf): The fast fuzzy-finder that provides the interface for this by ~ab~using the `--preview` flag.
- [`jq`](https://github.com/stedolan/jq)/[`gojq`](https://github.com/itchyny/gojq)/[`yq`](https://github.com/mikefarah/yq): The workhorse for processing query input. I typically use `gojq`, as it uses `jq` syntax exactly (unlike `yq`), and handles modules (I couldn't get them working with `jq`, ironically). If you want to use a different interpreter, pass it to `--jq-bin`. You may also want to pass `--no-default-args`, customize `--color-flag` and `--no-color-flag`, and pass any additional options as the last parameter. For example, if I were using `yq`, I could run it like this:

  ```console
  cat mydoc.xml | jq-repl --jq-bin yq --no-default-args -- --input-format xml --output-format json
  ```

- [`vd` (VisiData)](https://github.com/saulpw/visidata): A data explorer tool in the TUI. Bound to <kbd>alt</kbd>+<kbd>v</kbd> by default.
- [`bat`](https://github.com/sharkdp/bat): A pager with syntax highlighting. Bound by default to <kbd>alt</kbd>+<kbd>L</kbd>.
- [`less`](https://github.com/gwsw/less): A pager. Bound by default to <kbd>alt</kbd>+<kbd>l</kbd>.
- [`nvim` (Neovim)](https://github.com/neovim/neovim): A TUI editor based on Vim. Bound by default to <kbd>alt</kbd>+<kbd>e</kbd>. If you want to use a different editor, pass it to `--editor` (or set with `$EDITOR`), along with whatever options it you want with `--editor-options`. It needs to run in the foreground, and handle reading from standard input (`/dev/stdin`). For example, if you want to use Visual Studio Code, you can run this:

  ```console
  jq-repl --editor code --editor-options '--wait -' ./path/to/file.json
  ```

- [`gron`](https://github.com/tomnomnom/gron): Makes JSON greppable. Bound to <kbd>ctrl</kbd>+<kbd>space</kbd> by default. Toggle back with <kbd>alt</kbd>+<kbd>space</kbd>.

## License

Released under MIT (see [LICENSE](/LICENSE)). Note that any program called on by this is subject to its own license and terms[^2].

## Why?

I was learning jq (…I still am, but I was at the time too), and found the default repl cycle too slow, and wanted something more shortcut-friendly than <https://jqplay.org>. I discovered each of these tools over time (`fzf`, `jq`, `vd`, `gron`…) and thought "hey…", and the rest is history.

## Why Rust?

Because I wanted to. :grin:

I don't trust myself not to make a mistake in Bash, and I find it difficult to manage complex shell arguments and handle all the complex shell quoting[^3]. Besides, I don't know Python. :slightly_smiling_face:

## Inspiration

- [Implementing a jq REPL with fzf](https://gist.github.com/reegnz/b9e40993d410b75c2d866441add2cb55)

[^1]: Whatever _that_ means.
[^2]: I am not a lawyer, so if I'm making a mistake here, please let me know!
[^3]: I'm sure there are a lot of bugs hiding in how stuff is quoted that I'm not aware of. If you find any, let me know!
