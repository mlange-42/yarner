# Java example with Gradle

This example produces a minimalistic Java application. It serves to demonstrate how to create a Java Gradle project.

To build the project (i.e. generate code and documentation output), run the following command in the current directory:

```
> yarner
```

To run the Java project, navigate into sub-folder `code` and run with `gradlew`:

```
> cd code
> gradlew run
```

## Main class

We create a simple class `Main` in package `yarner_example`. We use a macro invocation to draw in the main method.

```java
//- file:src/main/java/yarner_example/Main.java
package yarner_example;

public class Main {
    // ==> Main method.
}
```

## Main method

The main method simply prints "Hello World!".

```java
//- Main method
public static void main(String[] args) {
    System.out.println("Hello World!");
}
```

## Gradle files

[Gradle](https://gradle.org/) is used as build tool and for dependency management.

```groovy
//- file:build.gradle
apply plugin: 'application'

repositories {
    mavenCentral()
}

dependencies {
   // add dependencies...
}

application {
    getMainClass().set('yarner_example.Main')
}
```

```groovy
//- file:settings.gradle
rootProject.name = 'YarnerExample'
```

The remaining files required by Gradle are stored in sub-directoy `gradle`. They are copied into the project through the settings in file `Yarner.toml`, section `[paths]` (see options `code_files` and `code_paths`).
