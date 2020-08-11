{
  description = "ykprovision";

  inputs = {
    nixpkgs = { url = "github:nixos/nixpkgs/nixos-unstable"; };
  };

  outputs = inputs:
    let
      nameValuePair = name: value: { inherit name value; };
      genAttrs = names: f: builtins.listToAttrs (map (n: nameValuePair n (f n)) names);
      forAllSystems = genAttrs [ "x86_64-linux" "i686-linux" "aarch64-linux" ];

      pkgsFor = pkgs: sys:
        import pkgs {
          system = sys;
          config = { allowUnfree = true; };
        };

      mkSystem = sys: pkgs_: hostname:
        pkgs_.lib.nixosSystem {
          system = sys;
          modules = [(./. + "/machines/${hostname}/configuration.nix")];
          specialArgs = {
            inputs = inputs;
            #secrets = import ./secrets;
          };
        };
    in rec {
      devShell = forAllSystems (system:
        (pkgsFor inputs.unstable system).mkShell {
          nativeBuildInputs = with (pkgsFor inputs.cmpkgs system); [
            (pkgsFor inputs.master system).nixFlakes
            (pkgsFor inputs.stable system).cachix
            bash cacert curl git jq mercurial
            nettools openssh ripgrep rsync
            nix-build-uncached nix-prefetch-git
            packet-cli
            sops
          ];
        }
      );

      # packages = // import nixpkgs, expose colePkgs

      nixosConfigurations = {
        azdev  = mkSystem "x86_64-linux" inputs.unstable "azdev";
        rpione = mkSystem "aarch64-linux" inputs.pipkgs "rpione";
        xeep   = mkSystem "x86_64-linux"  inputs.cmpkgs "xeep";
      };

      machines = {
        azdev = inputs.self.nixosConfigurations.azdev.config.system.build.azureImage;
        xeep = inputs.self.nixosConfigurations.xeep.config.system.build.toplevel;
        rpione = inputs.self.nixosConfigurations.rpione.config.system.build.toplevel;
      };

      defaultPackage = [
        inputs.self.nixosConfigurations.xeep.config.system.build.toplevel
        inputs.self.nixosConfigurations.rpione.config.system.build.toplevel
      ];

      cyclopsJobs = {
        # 1. provision an age1 key
        # 2. get cyclops's advertised age1 pubkey
        # 3. add to .sops.yml
        # 4. ./util.sh e

        # cyclops:
        # - /nix is shared, but only per-customer
        # - same story with the cache
        xeep-update = {
          triggers = {
            cron = "*/*"; # use systemd format?
          };
          secrets = [
            { name="id_ed25519";   sopsFile = ./secrets/encrypted/id_ed25519;   path = "$HOME/.ssh/id_ed25519"; }
            { name="cachix.dhall"; sopsFile = ./secrets/encrypted/cachix.dhall; path = "$HOME/.cachix/cachix.dhall"; }
          ];
          stages = [
            # TODO: we can make some of these steps generic+shared, yay nix
            { name="prep";          script="./prep.sh"; }
            { name="update";        script="./update.sh"; }
            { name="build";         script="./build.sh"; }
            { name="update-flakes"; script="./update-flakes.sh"; }
          ];
        };
      };
    };
}
