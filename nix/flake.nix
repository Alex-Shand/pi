{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    nixos-hardware = {
      url = "github:NixOS/nixos-hardware/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, nixos-hardware, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        # Unique properties of each node
        load-node = name:
          let
            nodes = import ./nodes.nix;
            node = nodes.${name};
          in node // { inherit name; };

        # Not committed
        load-secrets = name:
          let
            secrets = { }; # import ./secrets.nix;
            global = secrets.global;
            by-node = secrets.by-node.${name};
          in global // by-node;

        # Take a config and produce an image
        build-image = name:
          let
            node = load-node name;
            secrets = load-secrets name;
            nixos = nixpkgs.lib.nixosSystem {
              system = "aarch64-linux";
              specialArgs = { inherit node secrets nixpkgs nixos-hardware; };
              modules = [ ./configuration.nix ];
            };
          in {
            image = nixos.config.system.build.sdImage;
            kernel = nixos.config.boot.kernelPackages.kernel;
          };

        # Node definitions
        admire = build-image "admire";
        backup = build-image "backup";

      in {
        packages = {
          admire = admire.image;
          backup = backup.image;
          kernels = nixpkgs.legacyPackages.${system}.symlinkJoin {
            name = "kernels";
            paths = [ admire.kernel backup.kernel ];
          };
        };
      });
}
