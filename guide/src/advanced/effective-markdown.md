# Effective Markdown

This chapter gives advice on how to use Markdown effectively, with Yarner and in general. It lists tools like editors and explains how to create rendered documents. Further, it provides tips like how to render math formulas with Markdown.

[[_TOC_]]

## Why Markdown?

Markdown is a lightweight markup language for creating formatted text documents. It is designed for writing in a plaintext editor, and to be simple and well readable for humans in its source code form. With that, Markdown is easy to learn and use, as well as future-proof. To the minimal Markdown syntax, Yarner adds some simple syntax elements for Literate Programming that do not break rendering of the source documents.

Using different tools, Markdown documents can be easily converted to HTML or PDF, or to other text editing formats like LaTeX or even MS Word.

When working with Git and a Git forge like GitHub or GitLab, Markdown offers the additional advantage that `.md` files are rendered on repository pages. Thus, Markdown sources and the "compiled" documentation output can be presented there directly, or automatically published using e.g. GitHub Pages or GitLab Pages.

## Markdown editors

[**Notepad++**](https://notepad-plus-plus.org) is a versatile Open Source text editor. Syntax highlighting for Markdown is available as [plugin](https://github.com/Edditoria/markdown-plus-plus)

[**Atom**](https://atom.io/) is a powerful and extensible Open Source text editor. Markdown syntax highlighting is built-in. For a rendered Markdown preview, multiple plugins are available, e.g. [Markdown Preview Enhanced](https://atom.io/packages/markdown-preview-enhanced).

Most **IDEs** support Markdown syntax highlighting, and some even a rendered preview (potentially via a plugin).

[**MarkText**](https://marktext.app/) is a pure Markdown editor with "real-time preview" (WYSIWYG). It is Open Source, and designed to be minimalistic and distraction-free. It supports all Markdown features necessary for using it in Yarner projects.

[**Typora**](https://typora.io/) is similar to MarkText, but only freeware, not Open Source.

## Markdown conversion

[**Pandoc**](https://pandoc.org/) is a command line tool for conversion between different markup formats. Besides Markdown, it supports a vast range of other formats for conversion in both directions: HTML, LaTeX, WikiText, MS Word, OpenOffice, LibreOffice, ...

[**mdBook**](https://github.com/rust-lang/mdBook) is a command line tool to create online books from Markdown files (e.g. the book you are currently reading). For details on how to use it with Yarner, see chapter [mdBook with Yarner](./mdbook.md).

Dedicated **Markdown editors** like [MarkText](https://marktext.app/) and [Typora](https://typora.io/) provide functionality to export Markdown documents as HTML and PDF.

## Math formulas

Different renderers and platforms support different ways to write math formulas.

### GitLab

GitLab supports inline math, enclosed in <code>$\`...\`$</code>, e.g. <code>$\`E = m c^2\`$</code>.

Math blocks are possible in fenced code blocks with language `math`:

~~~markdown
```math
a^2+b^2=c^2
```
~~~

### GitHub

GitHub does unfortunately not support any math syntax in Markdown. As a workaround, GitHub's math rendering service can be used to include formulas as images.

```html
<img src="https://render.githubusercontent.com/render/math?math=<formula>">
<img src="https://render.githubusercontent.com/render/math?math=E = m c^2">
```

### MarkText

MarkText supports inline math, enclosed in <code>$...$</code>, e.g. <code>$E = m c^2$</code>. This is similar to GitLab, except for the missing backticks.

For math blocks, fenced code blocks with language `math` are supported, like in GitLab:

~~~markdown
```math
a^2+b^2=c^2
```
~~~

Further, TeX-like math blocks can be used:

```markdown
$$
a^2+b^2=c^2
$$
```

### Pandoc

Pandoc supports TeX-like inline math, surrounded by `$...$` as well as math blocks:

```markdown
$$
a^2+b^2=c^2
$$
```

### mdBook

mdBook requires to explicitly enable math support in the `book.toml` config file through

```toml
[output.html]
mathjax-support = true
```

Inline equations must be enclosed in `\\(...\\)`.

Math blocks use `\\[` and `\\]` as delimiters:

```markdown
\\[
a^2+b^2=c^2
\\]
```

## Diagrams

Some editors and platforms support [Mermaid](https://mermaid-js.github.io/mermaid/) graphs through code blocks with language `mermaid`:

~~~markdown
```mermaid
graph TD;
    A-->B;
    A-->C;
    B-->D;
    C-->D;
```
~~~

Mermaid is supported by GitLab, MarkText, Typora, and by mdBook through a pre-processor.

## Further reading

* The [Markdown Guide](https://www.markdownguide.org/)
