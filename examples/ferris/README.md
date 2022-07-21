This is an example of how you can make your own plugins in markdown-it.

### What is it

There are 3 different plugins here:

- inline rule - turns `(\/)` into `🦀` in *inline context* (i.e. inside other text)

- block rule - turns `(\/)-------(\/)` into
  [ferris.svg](https://upload.wikimedia.org/wikipedia/commons/0/0f/Original_Ferris.svg)
  in *block context* (i.e. it has to occupy the entire line)

 - core rule - counts the number of nodes created by the above two plugins and writes
   that number at the end of the document

It represents three stages of markdown processing (block elements, inline elements
and AST post-processing).

### How to use

`cargo run --example ferris`
