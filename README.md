# innit

Innit annoying when you have to setup a new PC?

## Example Config

```yaml
actions:
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
    cmd:
      - nvm install lts
      - sudo nvm use lts
    tags:
      - dev

  - action: package.install
    name: paint.net
    winget_id: dotPDN.PaintDotNet
    tags:
      - tools

  - action: file.link
    src: C:\dotfiles\settings.json
    dest: C:\Program Files\My App\settings.json
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
