# Configuration

fnm comes with many features out of the box. Some of them are opt-in because they change your shell's default behavior, while others are feature flags that let us ship improvements without breaking existing workflows.

These features can be configured by adding flags to the `fnm env` call when initializing the shell. For instance, if your shell setup looks like `eval "$(fnm env)"`, add a flag by changing it to `eval "$(fnm env --my-flag=value)"`.

Here’s a list of these features and capabilities:

### `--use-on-cd`

**✅ Highly recommended**

`--use-on-cd` appends shell hooks to `fnm env`'s output. When you change directories, fnm switches the Node.js version based on `.node-version`, `.nvmrc`, or `package.json#engines.node` when `--resolve-engines` is enabled.

This lets you avoid thinking about `fnm use`: `cd <DIR>` is enough.

### `--version-file-strategy=recursive`

**✅ Highly recommended**

Makes `fnm use` and `fnm install` take parent directories into account when looking for a version file ("dotfile") when no argument was given.

So, let's say we have the following directory structure:

```
repo/
├── package.json
├── .node-version <- with content: `20.0.0`
└── packages/
  └── my-package/ <- I am here
    └── package.json
```

And I'm running the following command:

```sh-session
repo/packages/my-package$ fnm use
```

Then fnm will switch to Node.js v20.0.0.

Without the explicit flag, the value is set to `local`, which will not traverse the directory tree and therefore will print:

```sh-session
repo/packages/my-package$ fnm use
error: Can't find version in dotfiles. Please provide a version manually to the command.
```

### `--corepack-enabled`

**🧪 Experimental**

Runs [`corepack enable`](https://nodejs.org/api/corepack.html#enabling-the-feature) when a new version of Node.js is installed. Experimental due to the fact Corepack itself is experimental.

### `--resolve-engines`

**Enabled by default**

Treats `package.json#engines.node` as a valid Node.js version file ("dotfile"). So, if you have a package.json with the following content:

```json
{
  "engines": {
    "node": ">=20 <21"
  }
}
```

Then:

- `fnm install` will install the latest satisfying Node.js 20.x version available in the Node.js dist server
- `fnm use` will use the latest satisfying Node.js 20.x version available on your system, or prompt to install if no version matched

Use `--resolve-engines=false` to disable this behavior.
