import { script } from "./shellcode/script.js"
import { PowerShell } from "./shellcode/shells.js"
import testNodeVersion from "./shellcode/test-node-version.js"
import describe from "./describe.js"

for (const shell of [PowerShell]) {
  describe(shell, () => {
    test(`manages versions with natural language`, async () => {
      await script(shell)
        .then(shell.env({}))
        .then(
          shell.call("fnm", ["ai", "'please install node v8.11.3 and use it'"]),
        )
        .then(testNodeVersion(shell, "v8.11.3"))
        .then(
          shell.scriptOutputContains(
            shell.call("fnm", ["ai", "'what version am I using'"]),
            "v8.11.3",
          ),
        )
        .then(
          shell.scriptOutputContains(
            shell.call("fnm", ["ai", "'show installed versions'"]),
            "v8.11.3",
          ),
        )
        .then(
          shell.scriptOutputContains(
            shell.call("fnm", ["api", "'check current environment config'"]),
            "'current version'",
          ),
        )
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
