# Fendesk
A desktop version of [fend](https://printfn.github.io/fend/), built using [tauri](https://tauri.app).
Originally almost identical to the web version, but now has several added features (see below).

### TODO
Ordered by probable priority
- [ ] Better autocomplete with variables
- [ ] Syntax highlighting

# Features
- Exchange rates
- Slight syntax highlighting!
- Input hints and autocompletion! (slightly minimal currently due to library constraints)
- Shortcuts!
  - <kbd>Ctrl+W</kbd> or <kbd>Ctrl+D</kbd> quits the program
  - <kbd>Ctrl+C</kbd> copies the current input (if nothing is selected)
    - This will be configurable in the future, with options like previous result, hint or input
  - <kbd>Ctrl+S</kbd> saves all the calculations you've run (can be limited to a certain amount) to a file. One line per calculation.
  - <kbd>Ctrl+O</kbd> loads a file saved that way, and runs all the calculations in it.
- A nice UI! (mostly taken from the website)
- A load of configurable settings for all your function needs!

Attribution: Settings cog taken from: https://game-icons.net/1x1/lorc/cog.html