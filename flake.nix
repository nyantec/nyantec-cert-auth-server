{
  description = "A web server for validating X.509 Client Certificates";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: {
    overlays.default = final: prev: {
      nyantec-cert-auth-server = final.callPackage ./. { };
    };
  } // flake-utils.lib.eachDefaultSystem (system: let
    pkgs = import nixpkgs {
      inherit system;
      overlays = [ self.overlays.default ];
    };
  in rec {
    packages = {
      inherit (pkgs) nyantec-cert-auth-server;
      default = packages.nyantec-cert-auth-server;
    };

    legacyPackages = pkgs;

    devShells.default = import ./shell.nix { inherit pkgs; };
  });
}
