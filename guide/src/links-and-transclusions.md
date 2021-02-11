# Links and transclusions

For larger projects, not only multiple code files are desirable, but also multiple Markdown source and/or documentation files.
This chapter explains two features serving that purpose.

[[_TOC_]]

## Link following

By prefixing relative links with `@` (by default, configurable), Yarner can be instructed to follow these links and include linked files in the build process. E.g. to include file `linked.md`, it can be linked from the main file like this:

```markdown
The file @[linked.md](linked.md) is also part of this project.
```

The prefix is stripped from documentation output. The above content is modified to

```markdown
The file [linked.md](linked.md) is also part of this project.
```

## Transclusions

A transclusion means that the content of an entire file is drawn into another file.
Transclusions are achieved by wrapping a file path or a relative link into `@{{<path>}}`.
Here are two examples of valid transclusions:

```markdown
@{{path/to/file.md}}
@{{[file.md](path/to/file.md)}}
```

In the documentation output, the transclusion is replaced by the content of the referenced file.

During transclusion, unnamed code blocks are renamed to produce code in the same output file as if the file was not transcluded, but "compiled" directly.
E.g. an unnamed code block in file `transcluded.rs.md` is be renamed to `file:transcluded.rs`.
Note the prefix `file:`. See chapter [Multiple code files](./multiple-code-files.md) for details.

Transclusions are processed before macro evaluation. Thus, code blocks from the transcluded document can be used in the transcluding document, and vice versa.

A transclusion should be the only thing in a line.

**Transclusions are recursive**, so transcluded files can also transculde other files themselves.

### Link correction

Relative links in transcluded files are corrected to still point to the correct target.
E.g., when including a file `path/file.md` into `README.md`,
a link to `file2.md` in that file would become `path/file2.md`
in the documentation output of `README.md`.
