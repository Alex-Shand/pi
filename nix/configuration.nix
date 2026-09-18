{ ... }: {
  system.stateVersion = "26.05";
  imports = [ ./boot.nix ./networking.nix ./locale.nix ./user.nix ./ssh.nix ];
}
