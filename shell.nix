{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "nyantec-cert-auth-server-dev-shell";
  buildInputs = with pkgs; [
    cargo
    openssl
    pkg-config
    rustc
    rustfmt
  ];
}
