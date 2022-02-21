
# ![protovend](docs/logo.png)

Protovend is a tool for managing your vendored protobuf files with ease. Simply install the tool and you can run it like git from the root of the project.

## Features

Protovend will 'vendor' another service's protobuf schema files into a local repository, so that they can be used to generate clients for it (using separate codegen tooling that suits the language and libraries in use in the local repository).

- Easier dependency management
- Repository caching and data caching for improved performance.
- Manage and install dependencies from Github.

## Example usage

In these examples, we'll be vendoring protos from `somegroup/producer-service` (a GitHub.com repository) into the local working copy for `consumer-service`. This will allow us to generate a client for `consumer-service` to use when talking to the deployed `producer-service`.

1. `protovend init`: Preparing a repository to receive vendored files:

   ```
   $ protovend init
   (INFO) Created /Users/me/consumer-service/.protovend.yml
   (INFO) Created /Users/me/consumer-service/.protovend.lock
   ```

2. `protovend add`: Adding a dependency declaration

   ```
   $ protovend add git@github.com:somegroup/producer-service.git
   (INFO) somegroup/producer-service added to protovend metadata
   ```

3. `protovend install`: Fetching and vendoring protos from the dependency, and generating a lockfile

   ```
   $ protovend install
   (INFO) Fetching latest commit hash from each new repo in github. Current: main@somegroup/producer-service
   (INFO) Fetching proto files from github repos. Current: main@somegroup/producer-service
   (INFO) Next Steps:
   Check the following protovend generated files and vendored proto directory (containing .proto files) into source control
     - .protovend.yml
     - .protovend.lock
     - vendor/proto
   ```

4. `protovend update <repo>`: Updating a single dependency

   ```
   $ protovend update somegroup/producer-service
   (INFO) Fetching latest commit hash from each new repo in github. Current: main@somegroup/producer-service
   (INFO) Fetching proto files from github repos. Current: main@somegroup/producer-service
   (INFO) Next Steps:
   Check the following protovend generated files and vendored proto directory (containing .proto files) into source control
     - .protovend.yml
     - .protovend.lock
     - vendor/proto
   ```

5. `protovend update`: Updating all tracked dependencies to latest

   ```
   $ protovend update
   (INFO) Fetching latest commit hash from each new repo in github. Current: main@somegroup/producer-service
   (INFO) Fetching proto files from github repos. Current: main@somegroup/producer-service
   (INFO) Next Steps:
   Check the following protovend generated files and vendored proto directory (containing .proto files) into source control
     - .protovend.yml
     - .protovend.lock
     - vendor/proto
   ```

## Transitive dependencies

Note that transitive dependencies between protobuf schemas are **explicitly not supported**.
That is to say, if your dependencies have their own dependencies on schemas in other repos, you are responsible for resolving them and vendoring each.

This design decision was mainly made to reduce the complexity of the tool, particularly when it comes to understanding versioning and potential version conflicts.
Given that deep/complex interdependencies in protobuf schema definitions are likely to be a code smell, we believe that developers should be able to easily resolve their transitive dependencies, and are best placed to do so.

## How it works

Protovend looks for a `/proto` folder in the repository that is being vendored, and copies all `*.proto` files found into `/vendor/proto` in the local repository.

When you run protovend in a project it generates a `.protovend.yml` and `.protovend.lock` file. These are configuration files that contain all of the information required for vendoring.

Both are designed to be 'human readable' and to be easily edited and required.

### `.protovend.yml`

This file contains a list of services that should be vendored. When `protovend add <repo>` is run, an entry is added here.

To remove a vendored service the service entry here should be removed.

#### Example `.protovend.yml`

```yml
min_protovend_version: 1.0.3
vendor:
  - branch: main
    repo: somegroup/producer-service
```

### `protovend.lock`

This file is generated during protovend install and protovend update operations.

It contains the commit id of the repo during the vendoring process.

#### Example `.protovend.lock`

```yml
imports:
  - branch: main
    commit: 6931b681ddea94753abb40105672c66d7e08d551
    repo: somegroup/producer-service
min_protovend_version: 1.0.3
updated: 2020-01-01 16:01:24.331398
```

The only time a commit id is changed is during an update.

### `./vendor/proto`

A directory that contains every protobuf file vendored.

This is wiped and re-generated during every `protovend install` and `protovend update`.

---

# Installation

TODO - once published to brew add steps here

## Usage

```sh
> protovend --help

Commands:
  add      Add a given git_group/git_repo to projects...
  cleanup  Delete all locally cached repos stored in...
  init     Initialise current directory with protovend...
  install  Install copies of protofiles declared in...
  lint     Lint function to ensure proto files are valid...
  update   Update one or all repos in protovend metadata...
```

### Troubleshooting

Run `protovend --help` to see all available commands.

You can provide `--debug` or `--info` param to see what is happening inside `protovend` like so:

```sh
protovend --debug COMMAND [ARGS]
```

## Developing

### Prerequisites

- Rust 1.37, Cargo

### Instructions

Build with

```bash
cargo build
```

Coding standards are maintained using the clippy and rustfmt tools. To run locally simply use:

```bash
cargo clippy
cargo fmt --all
```

## Contributing

To contribute please read our [guidelines](https://github.com/Skyscanner/protovend/blob/main/CONTRIBUTING.md).

## Attribution

The logo of Protovend is kindly provided by freepik: <a href="https://www.freepik.com/free-photos-vectors/food">Food vector created by macrovector - www.freepik.com</a>
