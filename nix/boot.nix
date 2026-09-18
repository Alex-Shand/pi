{ lib, node, nixpkgs, nixos-hardware, ... }:
let model = builtins.toString node.pi-model;
in {
  imports =
    [ "${nixpkgs}/nixos/modules/installer/sd-card/sd-image-aarch64.nix" ] ++ {
      "3" = [ nixos-hardware.nixosModules.raspberry-pi-3 ];
      "4" = [ nixos-hardware.nixosModules.raspberry-pi-4 ];
      "5" = [ nixos-hardware.nixosModules.raspberry-pi-5 ];
    }.${model};

  boot.supportedFilesystems.zfs = lib.mkForce false;
  hardware.raspberry-pi.firmware.uboot.enable = true;
}
