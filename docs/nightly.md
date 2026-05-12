# Working with Node nightly builds

fnm does not provide a shorthand for nightly builds. Use the
`--node-dist-mirror` flag to point fnm at the Node.js nightly distribution
mirror.

## List available nightly versions

```sh-session
$ fnm --node-dist-mirror https://nodejs.org/download/nightly/ ls-remote
```

## Install and use a nightly build

You can ask fnm to resolve a major version from the nightly mirror:

```sh-session
$ fnm --node-dist-mirror https://nodejs.org/download/nightly/ use 23 --install-if-missing
```

Or pass an exact nightly version from `ls-remote`:

```sh-session
$ fnm --node-dist-mirror https://nodejs.org/download/nightly/ use v23.0.0-nightly202407253de7a4c374 --install-if-missing
```

After the version is installed, it appears in the local version list:

```sh-session
$ fnm ls
```

You can then use it without passing `--node-dist-mirror` again:

```sh-session
$ fnm use 23
```
