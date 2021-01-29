# Continuous integration

This chapter shows some examples how to use Yarner with Continuous integration.
There are always two ways to get Yarner:

1. Use a rust image and install it using `cargo`
2. Download the [precompiled binaries](https://github.com/mlange-42/yarner/releases/)

In these examples, we use the second option.

[[_TOC_]]

## GitHub with Travis-CI

## GitHub Actions

## GitLab CI

In your GitLab project, create a file `.gitlab-ci-yml` with the following content:

```yml
variables:
  YARNER_VERSION: 0.2.2

before_script:
  - curl -L -o- https://github.com/mlange-42/yarner/releases/download/${YARNER_VERSION}/yarner-${YARNER_VERSION}-linux.tar.gz | tar xz
  - export PATH="$PWD/yarner:$PATH"

build:
  script:
    - yarner
  artifacts:
    paths:
      - docs/
      - code/
```

You should use the latest Yarner version for variable `YARNER_VERSION`.
