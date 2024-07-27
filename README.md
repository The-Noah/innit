# innit

Innit annoying when you have to setup a new PC?

## Example Config

```yaml
packages:
  - name: 1password
    winget_id: AgileBits.1Password
    tags:
      - core

  - name: 7zip
    winget_id: 7zip.7zip
    tags:
      - tools

  - name: nvm
    winget_id: CoreyButler.NVMforWindows
    cmd:
      - nvm install lts
      - sudo nvm use lts
    tags:
      - dev

  - name: paint.net
    winget_id: dotPDN.PaintDotNet
    tags:
      - tools
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
