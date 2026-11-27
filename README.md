# Rust Jeopardy

## About

This is a barebones application for running a Jeopardy-style
quiz game. Quiz definitions are loaded from XML files, so you
can create your own game without modifying the Rust code.

This is written in Rust, using the Bevy engine.

## Screenshots

![Untouched Board](https://i.imgur.com/FFuCEJ1.png)

![Clue Reading](https://i.imgur.com/9y9nY5R.png)

![Touched Board](https://i.imgur.com/HYUd1nL.png)

## Usage

This is a rather basic Rust setup, so usage should be straight-forward.

I will note for those looking at the code that this is my first Rust application, so if anything
seems off or strange then it probably just is.

Originally written with Rust 1.54.0, this project now uses the
Rust 2024 edition and Bevy 0.10.

Run from the project directory with `cargo run` to play the
bundled Studio Ghibli quiz (`assets/gibli-quiz.xml`). To select
a different quiz, pass its file path on the command line:

```sh
cargo run -- path/to/my-quiz.xml
```

### Quiz files

Use `assets/gibli-quiz.xml` as a template. A quiz has a `<game>`
root, six `<category>` elements in left-to-right order, and
five `<clue>` elements in each category, from $200 to $1000.
For example, one category of a game could look like this:

```xml
<game name="General Knowledge" author="Quiz writer">
  <category name="Geography">
    <clue>The capital of Oregon.</clue>
    <clue>This river flows through Portland.</clue>
    <clue>The highest mountain in Oregon.</clue>
    <clue>Oregon's largest lake by surface area.</clue>
    <clue>The county containing the state capital.</clue>
  </category>
  <!-- Add five more categories here. -->
</game>
```

The game name is optional and defaults to `Quiz!`. The author
attribute is optional metadata. Clue whitespace is preserved
for display: use line breaks to lay out long clues, `&#10;` for
line breaks in category names, and `&amp;` for ampersands.
Malformed XML or an incorrect number of categories or clues
is reported before the game window opens.

## Contact

Feel free to contact me at benjamin.w.massey@gmail.com with any questions / inquiries.
