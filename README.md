# innit

Innit annoying when you have to setup a new PC?

## Example Config

```yaml
actions:
  - action: package.install
    name: git
    winget_id: Git.Git
    tags:
      - core
      - dev

  - action: package.install
    name: 1password
    winget_id: AgileBits.1Password
    tags:
      - core

  - action: package.install
    name: 7zip
    winget_id: 7zip.7zip
    tags:
      - tools

  - action: package.install
    name: nvm
    winget_id: CoreyButler.NVMforWindows
    tags:
      - dev

  - action: command.run
    command: nvm install lts

  - action: command.run
    command: sudo nvm use lts

  - action: github.repo
    repo: The-Noah/dotfiles
    dest: "{{ user.home }}"

  - action: file.link
    name: Windows Terminal
    src: "{{ user.home }}/dotfiles/windows_terminal.json"
    dest: "{{ user.home }}/AppData/Local/Packages/Microsoft.WindowsTerminal_8wekyb3d8bbwe/LocalState/settings.json"
```

## Usage

Install everything:

```bash
innit config.yaml
```

Install only packages tagged with `tools`:

```bash
innit config.yaml -t tools
```
