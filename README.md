# innit

Innit annoying when you have to setup a new PC?

## Installation

```bash
cargo install innit
```

## Usage

The configuration file is automatically loaded from either `$HOME/.config/innit.yaml` or `$HOME/dotfiles/.config/innit.yaml`.

Run all actions:

```bash
innit
```

Only run actions tagged with `dotfiles`:

```bash
innit -t dotfiles
```

If you need to use a custom path for the config:

```bash
innit -c /path/to/my/config.yaml
```

## Example Config

```yaml
actions:
  - action: package.install
    name: git
    winget_id: Git.Git
    tags: [core, dev]
    platforms: [windows]

  - action: package.install
    name: git lfs
    winget_id: GitHub.GitLFS
    tags: [dev]
    platforms: [windows]

  - action: package.install
    name: 7zip
    winget_id: 7zip.7zip
    tags: [tools]
    platforms: [windows]

  - action: file.download
    url: https://github.com/ryanoasis/nerd-fonts/releases/download/v3.2.1/0xProto.zip
    dest: "{{ user.home }}/Downloads/0xProto.zip"

  - action: github.repo
    repo: The-Noah/dotfiles
    dest: "{{ user.home }}"
    tags: [dotfiles]

  - action: file.link
    name: Innit
    src: "{{ user.home }}/dotfiles/.config/innit.yaml"
    dest: "{{ user.home }}/.config/innit.yaml"
    tags: [dotfiles]

  - action: file.link
    name: Windows Terminal
    src: "{{ user.home }}/dotfiles/windows_terminal.json"
    dest: "{{ user.home }}/AppData/Local/Packages/Microsoft.WindowsTerminal_8wekyb3d8bbwe/LocalState/settings.json"
    tags: [dotfiles]
    platforms: [windows]

  - action: file.link
    name: Starship
    src: "{{ user.home }}/dotfiles/.config/starship.toml"
    dest: "{{ user.home }}/.config/starship.toml"
    tags: [dotfiles]

  - action: file.link
    name: timr
    src: "{{ user.home }}/dotfiles/.config/timr.toml"
    dest: "{{ user.home }}/.config/timr.toml"
    tags: [dotfiles]

  - action: file.link
    name: timr
    src: "{{ user.home }}/dotfiles/.hushlogin"
    dest: "{{ user.home }}/.hushlogin"
    tags: [dotfiles]
    platforms: [macos]
```

## License

[MIT](LICENSE)
