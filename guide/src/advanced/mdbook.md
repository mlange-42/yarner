# mdBook with Yarner

[[_TOC_]]

```plaintext
project
  |
  |-- book/        <-------.       <rendered book>
  |                        |
  |-- code/                |       <code output>
  |     '-- ...         <--|--.
  |                        |  |
  |-- docs/                |  |
  |     |-- SUMMARY.md  ---'  |    <book sources>
  |     '-- capter-1.md <-----|
  |                           |
  |-- lp/                     |
  |     |-- SUMMARY.md  ------'    <yarner sources>
  |     '-- capter-1.md
  |
  |-- book.toml
  '-- Yarner.toml
```

```toml
[parser]
...

[paths]
root = "lp/"
code = "../code/"
docs = "../docs/"

files = ["SUMMARY.md"]
...
```
