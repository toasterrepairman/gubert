{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, ... }@inputs:
    utils.lib.eachDefaultSystem
      (system:
        let
          name = "imgread";
          pkgs = nixpkgs.legacyPackages.${system};
        in
        rec {
          packages.${name} = pkgs.callPackage ./default.nix {
            inherit (inputs);
          };

          # `nix build`
          defaultPackage = packages.${name};

          # `nix run`
          apps.${name} = utils.lib.mkApp {
            inherit name;
            drv = packages.${name};
          };
          defaultApp = packages.${name};

          # `nix develop`
          devShells = {
            default = pkgs.mkShell {
              nativeBuildInputs =
                with pkgs; [
                  rustc
                  cargo
                  cairo
                  openssl
                  pkg-config
                  git
		          # for GTK
		          cairo
		          gdk-pixbuf
		          atk
		          gobject-introspection
		          graphene
		          gtk3.dev
		          gtksourceview5
		          libadwaita
		          openssl_legacy.dev
		          pandoc
		          pango
		          pkg-config
		          appstream-glib
		          polkit
		          gettext
		          desktop-file-utils
		          meson
		          git
		          wrapGAppsHook4
                ];
            };
          };
        }
      );
}
