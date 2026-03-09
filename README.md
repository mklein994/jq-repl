# jq-repl

An interactive JSON explorer.

This is essentially a mix of shell tools that are useful for exploring JSON documents glued together to produce an interactive tool for transforming, viewing, and otherwise exploring JSON.

The gist of this program comes from this:

```console
$ fzf --disabled --preview 'gojq {q} <file.json>'
```

That's the core of this program. Everything else is to make this process nicer: better file handling, dynamic shortcuts, default arguments, and so on.

> \[!WARNING]
>
> This project is highly experimental, built to scratch an itch, so it might not be right for you. Consider this project unstable until the 1.0 release, as there may be breaking changes any time I feel like it. That being said, I've been using this tool for a number of years now, and it's been game-changing, so it might be worth trying out yourself.
>
> See also: <https://hannahilea.com/blog/houseplant-programming/>

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
- [`jq`](https://github.com/jqlang/jq)/[`gojq`](https://github.com/itchyny/gojq)/[`yq`](https://github.com/mikefarah/yq): The workhorse for processing query input. I typically use `gojq`, as it uses `jq` syntax exactly (unlike `yq`), and handles modules (I couldn't get them working with `jq`, ironically). If you want to use a different interpreter, pass it to `--jq-bin`. You may also want to pass `--no-default-args`, customize `--color-flag` and `--no-color-flag`, and pass any additional options as the last parameter. For example, if I were using `yq`, I could run it like this:

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

Released under MIT (see [LICENSE](/LICENSE)). Note that any program called on by this is subject to its own license and terms[^1].

## Why?

I created this tool because I wanted to learn jq faster. I started with `echo '{ "some": "json" }' | jq 'foo | map(bar)'`, but found that to be too slow for learning: I wanted to see the results of what I typed _now_, without having to scroll through the output or hit save in an editor. In my search for a good tool, I came across [this Hacker News comment](https://news.ycombinator.com/item?id=32909793) and decided to build around that myself.

## Why Rust?

Because I wanted to. :grin:

I don't trust myself to do this correctly in Bash, and I find it difficult to manage complex shell arguments and handle all the shell quoting. :slightly_smiling_face:

## Inspiration

- [Implementing a jq REPL with fzf](https://gist.github.com/reegnz/b9e40993d410b75c2d866441add2cb55)

## Wait… isn't this a TUI, not a REPL?

Yeah, about that… to be honest, I didn't know the difference until much later. I feel like sticking with the current name, since it's muscle memory now, but for the pedantics out there, I appologize.

[^1]: I am not a lawyer, so if I'm making a mistake here, please let me know!
