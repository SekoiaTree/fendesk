# Fendesk
A desktop version of [fend](https://printfn.github.io/fend/), built using [tauri](https://tauri.app).
Originally almost identical to the web version, but now has several added features (see below).

### TODO
Ordered by probable priority
- [ ] Add extra commands like Ctrl-S, Ctrl-O
- [ ] Add options like config-defined functions
- [ ] Better autocomplete with variables
- [ ] Syntax highlighting

# Features
- Exchange rates
- Slight syntax highlighting!
- Input hints and autocompletion! (slightly minimal currently due to library constraints)
- Shortcuts!
  - Ctrl-W quits the program (will be configurable; can also be Ctrl-D or both)
  - Ctrl-C copies the current input (if nothing is selected)
    - This will be configurable in the future, with options like previous result, hint or input
- A nice UI! (mostly taken from the website)

Attribution: Settings cog taken from: https://game-icons.net/1x1/lorc/cog.html