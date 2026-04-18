# xiny - Learn X in Y Minutes, in Your Terminal

Ever find yourself constantly tabbing over to [learnxinyminutes.com](https://learnxinyminutes.com/) to look up syntax you keep forgetting? Yeah, me too. So I made a CLI for it.

xiny clones the entire [learnxinyminutes-docs](https://github.com/adambard/learnxinyminutes-docs) repository locally and lets you read any of it offline, rendered with your preferred Markdown viewer. 187 subjects, 38 languages, all of it available from your terminal. The only hard dependency is `git`.

## Install

```bash
cargo install xiny
```

## Usage

```bash
# The basics - just tell it what you want to read
xiny python
xiny haskell
xiny bash

# Read it in another language!
xiny python -L ja-jp
xiny rust -L de-de

# What's available?
xiny --list                # All subjects
xiny --list -L de-de       # Subjects available in German
xiny --langs               # All available languages

# Just give me the file path, I'll handle the rest
xiny rust --where

# Keep the database up to date
xiny --sync                # Pull if behind
xiny --reclone             # Nuke and reclone
xiny --check-remote        # Check without pulling
```

## Viewer
xiny renders the document using whatever Markdown viewer you configure. I recommend [glow](https://github.com/charmbracelet/glow) for the best experience, but anything that can render Markdown works -- `bat`, `mdt`, whatever you prefer.

```bash
xiny --set-conf renderer glow
```

If no renderer is set, you get the raw Markdown. Still readable, just not as pretty.

## Config

There's a small config file you can poke at from the CLI:

```bash
xiny --get-conf              # Dump everything
xiny --get-conf renderer     # Get a specific key
xiny --set-conf renderer glow
```

## Shell Completions

Subject names and language tags are baked into the completions, so tab completion works for everything. Generate and source them for your shell:

```bash
# Bash
xiny --gencompletions bash >> ~/.bash_completion

# Zsh
xiny --gencompletions zsh > ~/.zfunc/_xiny

# Fish
xiny --gencompletions fish > ~/.config/fish/completions/xiny.fish
```

## First Run

On first use, xiny will ask to clone the documentation database (about 50MB). It goes into `~/.local/share/xiny` by default. After that, everything is offline.

## Building from Source

```bash
git clone https://github.com/PsychedelicShayna/xiny
cd xiny
cargo build --release
```

## License

GPL-3.0-or-later
