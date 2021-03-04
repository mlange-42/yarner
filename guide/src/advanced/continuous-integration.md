# Continuous integration

This chapter shows some examples how to use Yarner with Continuous integration, and how to publish Literate Programming documents.

[[_TOC_]]

## GitHub with Travis-CI

## GitHub Actions

## GitLab CI

In your GitLab project, create a file `.gitlab-ci-yml` with the following content:

```yml
image: ubuntu:latest

variables:
  YARNER_VERSION: 0.3.0

before_script:
  - apt-get update; apt-get -y install curl
  - curl -L -o- https://github.com/mlange-42/yarner/releases/download/${YARNER_VERSION}/yarner-${YARNER_VERSION}-linux-amd64.tar.gz | tar xz
  - export PATH="$PWD:$PATH"

build:
  script:
    - yarner --clean
  artifacts:
    paths:
      - docs/
      - code/
```

You should use the latest Yarner version for variable `YARNER_VERSION`.
