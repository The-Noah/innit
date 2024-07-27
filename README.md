# innit

Innit annoying when you have to setup a new PC?

## Config

```yaml
packages:
  - name: 1password
    winget_id: AgileBits.1Password

  - name: 7zip
    winget_id: 7zip.7zip

  - name: nvm
    winget_id: CoreyButler.NVMforWindows
    cmd:
      - nvm install lts
      - sudo nvm use lts

  - name: paint.net
    winget_id: dotPDN.PaintDotNet
```
