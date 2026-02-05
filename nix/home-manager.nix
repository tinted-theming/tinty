self: {...}: {
  imports = [
    (import ./modules/home/tinty/core.nix self)
    ./modules/home/tinty/terminal.nix
    ./modules/home/tinty/fzf.nix
    ./modules/home/tinty/shell.nix
    ./modules/home/tinty/tmux.nix
    ./modules/home/tinty/delta.nix
  ];
}
